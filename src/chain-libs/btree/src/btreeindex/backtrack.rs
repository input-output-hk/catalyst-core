use super::transaction::{MutablePage, PageRef, PageRefMut, WriteTransaction};
use super::{transaction};
use crate::btreeindex::{
    borrow::{Immutable, Mutable},
    node::{NodeRef, NodeRefMut},
    page_manager::PageManager,
    Node, PageHandle, PageId, Pages,
};
use crate::mem_page::MemPage;
use crate::Key;




use std::convert::{TryFrom, TryInto};
use std::marker::PhantomData;



/// this is basically a stack, but it will rename pointers and interact with the builder in order to reuse
/// already cloned pages
pub struct InsertBacktrack<'txbuilder, 'txmanager: 'txbuilder, 'storage: 'txmanager, K>
where
    K: Key,
{
    tx: &'txbuilder mut transaction::WriteTransaction<'txmanager, 'storage>,
    backtrack: Vec<PageId>,
    new_root: Option<PageId>,
    phantom_key: PhantomData<[K]>,
    key_buffer_size: u32,
}

/// this is basically a stack, but it will rename pointers and interact with the builder in order to reuse
/// already cloned pages
pub struct DeleteBacktrack<'txbuilder, 'txmanager: 'txbuilder, 'storage: 'txmanager, K>
where
    K: Key,
{
    tx: &'txbuilder transaction::WriteTransaction<'txmanager, 'storage>,
    backtrack: Vec<PageId>,
    parent_info: Vec<(Option<usize>, Option<PageId>, Option<PageId>)>,
    new_root: Option<PageId>,
    phantom_key: PhantomData<[K]>,
    key_buffer_size: u32,
}

pub struct DeleteNextElement<'a, 'b: 'a, 'c: 'b, 'd: 'c, K>
where
    K: Key,
{
    pub next: PageRefMut<'a, 'd>,
    pub parent: Option<PageRefMut<'a, 'd>>,
    pub anchor: Option<usize>,
    pub left: Option<PageRef<'a, 'd>>,
    pub right: Option<PageRef<'a, 'd>>,
    backtrack: &'a mut DeleteBacktrack<'b, 'c, 'd, K>,
}

impl<'a, 'b: 'a, 'c: 'b, 'd: 'c, K> DeleteNextElement<'a, 'b, 'c, 'd, K>
where
    K: Key,
{
    pub fn mut_left_sibling(&self, key_size: usize) -> PageRefMut<'a, 'd> {
        let left_id = self.left.as_ref().unwrap().id();
        match self.backtrack.tx.mut_page(dbg!(left_id)).unwrap() {
            MutablePage::InTransaction(handle) => handle,
            MutablePage::NeedsParentRedirect(redirect_pointers) => {
                redirect_pointers.redirect_parent_in_tx::<K>(key_size, self.parent.clone().unwrap())
            }
        }
    }

    pub fn mut_right_sibling(&self, key_size: usize) -> PageRefMut<'a, 'd> {
        let right_id = self.right.as_ref().unwrap().id();
        match self.backtrack.tx.mut_page(dbg!(right_id)).unwrap() {
            MutablePage::InTransaction(handle) => handle,
            MutablePage::NeedsParentRedirect(redirect_pointers) => {
                redirect_pointers.redirect_parent_in_tx::<K>(key_size, self.parent.clone().unwrap())
            }
        }
    }

    /// delete current node, this just adds the id to the list of free pages *after* the transaction is confirmed
    pub fn delete_node(&self) {
        let id = self.next.id();
        self.backtrack.delete_node(id)
    }

    /// delete right sibling of current node, this just adds the id to the list of free pages *after* the transaction is confirmed
    pub fn delete_right_sibling(&self) {
        let id = self.right.as_ref().map(|handle| handle.id()).unwrap();
        self.backtrack.delete_node(dbg!(id))
    }

    pub fn set_root(&self, id: PageId) {
        self.backtrack.tx.current_root.set(id)
    }
}

impl<'txbuilder, 'txmanager: 'txbuilder, 'storage: 'txmanager, K>
    DeleteBacktrack<'txbuilder, 'txmanager, 'storage, K>
where
    K: Key,
{
    pub(crate) fn new_search_for(
        tx: &'txbuilder mut WriteTransaction<'txmanager, 'storage>,
        key: &K,
    ) -> Self {
        let key_buffer_size = tx.key_buffer_size.clone();
        let mut backtrack = DeleteBacktrack {
            tx,
            backtrack: vec![],
            parent_info: vec![],
            new_root: None,
            phantom_key: PhantomData,
            key_buffer_size,
        };
        backtrack.search_for(key);
        backtrack
    }
    /// traverse the tree while storing the path, so we can then backtrack while splitting
    pub fn search_for(&mut self, key: &K) {
        enum Step {
            Internal(Option<usize>, Option<PageId>, Option<PageId>),
            Leaf,
        }

        let mut current = self.tx.root();

        loop {
            let page = self.tx.get_page(current).unwrap();

            let found_leaf = page.as_node(
                self.key_buffer_size.try_into().unwrap(),
                |node: Node<K, &[u8]>| {
                    if let Some(inode) = node.try_as_internal() {
                        let upper_pivot = match inode.keys().binary_search(key) {
                            Ok(pos) => Some(pos + 1),
                            Err(pos) => Some(pos),
                        }
                        .filter(|pos| pos < &inode.children().len());

                        let anchor = upper_pivot
                            .or_else(|| inode.keys().len().checked_sub(1))
                            .and_then(|up| up.checked_sub(1));

                        let left_sibling_id = anchor.and_then(|pos| inode.children().try_get(pos));

                        let right_sibling_id = anchor
                            .map(|pos| pos + 2)
                            .or(Some(1))
                            .and_then(|pos| inode.children().try_get(pos));

                        if let Some(upper_pivot) = upper_pivot {
                            current = inode.children().get(upper_pivot);
                        } else {
                            let last = inode.children().len().checked_sub(1).unwrap();
                            current = inode.children().get(last);
                        }

                        Step::Internal(anchor, left_sibling_id, right_sibling_id)
                    } else {
                        Step::Leaf
                    }
                },
            );

            self.backtrack.push(page.id());

            match found_leaf {
                Step::Internal(anchor, left, right) => self.parent_info.push((anchor, left, right)),
                Step::Leaf => break,
            }
        }
    }

    pub fn get_next<'this>(
        &'this mut self,
    ) -> Result<Option<DeleteNextElement<'this, 'txbuilder, 'txmanager, 'storage, K>>, std::io::Error>
    {
        let id = match dbg!(&mut self.backtrack).pop() {
            Some(id) => id,
            None => return Ok(None),
        };

        if self.backtrack.is_empty() {
            assert!(self.new_root.is_none());
            self.new_root = Some(id);
        }

        let key_buffer_size = usize::try_from(self.key_buffer_size).unwrap();

        let (parent, anchor, left, right) = if self.backtrack.is_empty() {
            (None, None, None, None)
        } else {
            let parent = self.backtrack.last().unwrap();

            let (anchor, left, right) = self.parent_info.pop().unwrap();
            (Some(parent), anchor, left, right)
        };

        let next = match self.tx.mut_page(id)? {
            transaction::MutablePage::NeedsParentRedirect(rename_in_parents) => {
                // this part may be tricky, we need to recursively clone and redirect all the path
                // from the root to the node we are writing to. We need the backtrack stack, because
                // that's the only way to get the parent of a node (because there are no parent pointers)
                // so we iterate it in reverse but without consuming the stack (as we still need it for the
                // rest of the insertion algorithm)
                let mut rename_in_parents = Some(rename_in_parents);
                let mut finished = None;
                for id in self.backtrack.iter().rev() {
                    let result = rename_in_parents
                        .take()
                        .unwrap()
                        .redirect_parent_pointer::<K>(key_buffer_size, *id)?;

                    match result {
                        MutablePage::NeedsParentRedirect(rename) => {
                            rename_in_parents = Some(rename)
                        }
                        MutablePage::InTransaction(handle) => {
                            finished = Some(handle);
                            break;
                        }
                    }
                }
                match finished {
                    Some(handle) => handle,
                    None => rename_in_parents.unwrap().finish(),
                }
            }
            transaction::MutablePage::InTransaction(handle) => handle,
        };

        let left = left.and_then(|id| self.tx.get_page(id));
        let right = right.and_then(|id| self.tx.get_page(id));
        let parent = parent.map(|id| match self.tx.mut_page(*id).unwrap() {
            MutablePage::InTransaction(handle) => handle,
            _ => unreachable!(),
        });

        Ok(Some(DeleteNextElement {
            next,
            parent,
            anchor,
            left,
            right,
            backtrack: self,
        }))
    }

    pub fn delete_node(&self, page_id: PageId) {
        self.tx.delete_node(page_id)
    }
}

impl<'txbuilder, 'txmanager: 'txbuilder, 'index: 'txmanager, K>
    InsertBacktrack<'txbuilder, 'txmanager, 'index, K>
where
    K: Key,
{
    pub(crate) fn new_search_for(
        tx: &'txbuilder mut WriteTransaction<'txmanager, 'index>,
        key: &K,
    ) -> Self {
        let key_buffer_size = tx.key_buffer_size.clone();
        let mut backtrack = InsertBacktrack {
            tx,
            backtrack: vec![],
            new_root: None,
            phantom_key: PhantomData,
            key_buffer_size,
        };

        backtrack.search_for(key);
        backtrack
    }
    /// traverse the tree while storing the path, so we can then backtrack while splitting
    fn search_for<'a>(&'a mut self, key: &K) {
        let mut current = self.tx.root();

        loop {
            let page = self.tx.get_page(current).unwrap();

            let found_leaf = page.as_node(
                self.key_buffer_size.try_into().unwrap(),
                |node: Node<K, &[u8]>| {
                    if let Some(inode) = node.try_as_internal() {
                        let upper_pivot = match inode.keys().binary_search(key) {
                            Ok(pos) => Some(pos + 1),
                            Err(pos) => Some(pos),
                        }
                        .filter(|pos| pos < &inode.children().len());

                        if let Some(upper_pivot) = upper_pivot {
                            current = inode.children().get(upper_pivot);
                        } else {
                            let last = inode.children().len().checked_sub(1).unwrap();
                            current = inode.children().get(last);
                        }
                        false
                    } else {
                        true
                    }
                },
            );

            self.backtrack.push(page.id());

            if found_leaf {
                break;
            }
        }
    }

    pub fn get_next<'a>(&'a mut self) -> Result<Option<PageRefMut<'a, 'index>>, std::io::Error> {
        let id = match self.backtrack.pop() {
            Some(id) => id,
            None => return Ok(None),
        };

        if self.backtrack.is_empty() {
            assert!(self.new_root.is_none());
            self.new_root = Some(id);
        }

        let key_buffer_size = usize::try_from(self.key_buffer_size).unwrap();

        match self.tx.mut_page(id)? {
            transaction::MutablePage::NeedsParentRedirect(rename_in_parents) => {
                // this part may be tricky, we need to recursively clone and redirect all the path
                // from the root to the node we are writing to. We need the backtrack stack, because
                // that's the only way to get the parent of a node (because there are no parent pointers)
                // so we iterate it in reverse but without consuming the stack (as we still need it for the
                // rest of the insertion algorithm)
                let mut rename_in_parents = rename_in_parents;
                for id in self.backtrack.iter().rev() {
                    let result =
                        rename_in_parents.redirect_parent_pointer::<K>(key_buffer_size, *id)?;

                    match result {
                        MutablePage::NeedsParentRedirect(rename) => rename_in_parents = rename,
                        MutablePage::InTransaction(handle) => return Ok(Some(handle)),
                    }
                }
                let page = rename_in_parents.finish();
                Ok(Some(page))
            }
            transaction::MutablePage::InTransaction(handle) => Ok(Some(handle)),
        }
    }

    pub fn has_next(&self) -> bool {
        self.backtrack.last().is_some()
    }

    pub fn add_new_node(
        &mut self,
        mem_page: MemPage,
        key_buffer_size: u32,
    ) -> Result<PageId, std::io::Error> {
        self.tx.add_new_node(mem_page, key_buffer_size)
    }

    pub fn new_root(
        &mut self,
        mem_page: MemPage,
        key_buffer_size: u32,
    ) -> Result<(), std::io::Error> {
        let id = self.tx.add_new_node(mem_page, key_buffer_size)?;
        self.new_root = Some(id);

        Ok(())
    }
}

impl<'txbuilder, 'txmanager: 'txbuilder, 'storage: 'txmanager, K> Drop
    for InsertBacktrack<'txbuilder, 'txmanager, 'storage, K>
where
    K: Key,
{
    fn drop(&mut self) {
        if let Some(new_root) = self.new_root {
            self.tx.current_root.set(new_root);
        } else {
            self.tx.current_root.set(*self.backtrack.first().unwrap());
        }
    }
}

impl<'txbuilder, 'txmanager: 'txbuilder, 'storage: 'txmanager, K> Drop
    for DeleteBacktrack<'txbuilder, 'txmanager, 'storage, K>
where
    K: Key,
{
    fn drop(&mut self) {
        if let Some(new_root) = self.new_root {
            self.tx.current_root.set(new_root);
        } else {
            self.tx.current_root.set(*self.backtrack.first().unwrap());
        }
    }
}
