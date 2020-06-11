/// Helpers to keep track of parent pointers and siblings when traversing the tree.
use super::transaction;
use super::transaction::{MutablePage, PageRef, PageRefMut, WriteTransaction};
use crate::btreeindex::node::{InternalNode, NodeRefMut};
use crate::btreeindex::{node::NodeRef, Node, PageId};
use crate::mem_page::MemPage;
use crate::FixedSize;
use std::marker::PhantomData;

/// this is basically a stack, but it will rename pointers and interact with the transaction in order to reuse
/// already cloned pages
pub struct InsertBacktrack<'txbuilder, 'txmanager: 'txbuilder, 'storage: 'txmanager, K>
where
    K: FixedSize,
{
    tx: &'txbuilder mut transaction::WriteTransaction<'txmanager, 'storage>,
    backtrack: Vec<PageId>,
    new_root: Option<PageId>,
    phantom_key: PhantomData<[K]>,
}

/// this is basically a stack, but it will rename pointers and interact with the transaction in order to reuse
/// already cloned pages
pub struct DeleteBacktrack<'txbuilder, 'txmanager: 'txbuilder, 'storage: 'txmanager, K>
where
    K: FixedSize,
{
    tx: &'txbuilder transaction::WriteTransaction<'txmanager, 'storage>,
    backtrack: Vec<PageId>,
    // The first parameter is the anchor used to get from the parent to the node in the top of the stack
    // the other two are both its siblings. The parent id can be found after the top of the stack, of course.
    parent_info: Vec<(Option<usize>, Option<PageId>, Option<PageId>)>,
    new_root: Option<PageId>,
    phantom_key: PhantomData<[K]>,
}

pub struct DeleteNextElement<'a, 'b: 'a, 'c: 'b, 'd: 'c, K>
where
    K: FixedSize,
{
    pub next_element: NextElement<'a, 'b, 'c, 'd, K>,
    pub mut_context: Option<MutableContext<'a, 'b, 'c, 'd, K>>,
}

/// this is basically a stack, but it will rename pointers and interact with the transaction in order to reuse
/// already cloned pages
pub struct UpdateBacktrack<'txbuilder, 'txmanager: 'txbuilder, 'storage: 'txmanager, K>
where
    K: FixedSize,
{
    tx: &'txbuilder mut transaction::WriteTransaction<'txmanager, 'storage>,
    backtrack: Vec<PageId>,
    key_to_update: K,
    new_root: Option<PageId>,
    phantom_key: PhantomData<[K]>,
}

// lifetimes on this are a bit bothersome with four 'linear' (re) borrows, there may be some way of refactoring this things, but that would probably need to be done higher in
// the hierarchy
/// type to operate on the current element in the stack (branch) of nodes. This borrows the backtrack and acts as a proxy, in order to make borrowing simpler, because
// XXX: having a left sibling means anchor is not None, and having a sibling in general means parent is not None also, maybe this invariants could be expressed in the type structure
pub struct NextElement<'a, 'b: 'a, 'c: 'b, 'd: 'c, K>
where
    K: FixedSize,
{
    pub next: PageRefMut<'a>,
    // anchor is an index into the keys array of a node used to find the current node in the parent without searching. The leftmost(lowest) child has None as anchor
    // this means it's inmediate right sibling would have anchor of 0, and so on.
    pub anchor: Option<usize>,
    pub left: Option<PageRef<'a>>,
    pub right: Option<PageRef<'a>>,
    backtrack: &'a DeleteBacktrack<'b, 'c, 'd, K>,
}

pub struct MutableContext<'a, 'b: 'a, 'c: 'b, 'd: 'c, K>
where
    K: FixedSize,
{
    parent: PageRefMut<'a>,
    // anchor is an index into the keys array of a node used to find the current node in the parent without searching. The leftmost(lowest) child has None as anchor
    // this means it's inmediate right sibling would have anchor of 0, and so on.
    current_id: PageId,
    left_id: Option<PageId>,
    right_id: Option<PageId>,
    backtrack: &'a DeleteBacktrack<'b, 'c, 'd, K>,
}

impl<'a, 'b: 'a, 'c: 'b, 'd: 'c, K> MutableContext<'a, 'b, 'c, 'd, K>
where
    K: FixedSize,
{
    pub fn mut_left_sibling(&mut self) -> (PageRefMut<'a>, &mut PageRefMut<'a>) {
        let sibling = match self.backtrack.tx.mut_page(self.left_id.unwrap()).unwrap() {
            MutablePage::InTransaction(handle) => handle,
            MutablePage::NeedsParentRedirect(redirect_pointers) => {
                redirect_pointers.redirect_parent_in_tx::<K>(&mut self.parent)
            }
        };

        (sibling, &mut self.parent)
    }

    pub fn mut_right_sibling(&mut self) -> (PageRefMut<'a>, &mut PageRefMut<'a>) {
        let sibling = match self.backtrack.tx.mut_page(self.right_id.unwrap()).unwrap() {
            MutablePage::InTransaction(handle) => handle,
            MutablePage::NeedsParentRedirect(redirect_pointers) => {
                redirect_pointers.redirect_parent_in_tx::<K>(&mut self.parent)
            }
        };

        (sibling, &mut self.parent)
    }

    /// delete right sibling of current node, this just adds the id to the list of free pages *after* the transaction is confirmed
    pub fn delete_right_sibling(&self) -> Result<(), ()> {
        match self.right_id {
            None => Err(()),
            Some(right_id) => {
                self.backtrack.delete_node(right_id);
                Ok(())
            }
        }
    }

    /// delete current node, this just adds the id to the list of free pages *after* the transaction is confirmed
    pub fn delete_node(&self) {
        self.backtrack.delete_node(self.current_id)
    }
}

impl<'a, 'b: 'a, 'c: 'b, 'd: 'c, K> NextElement<'a, 'b, 'c, 'd, K>
where
    K: FixedSize,
{
    pub fn set_root(&self, id: PageId) {
        self.backtrack.tx.current_root.set(id)
    }
}

enum Step<'a, K> {
    Leaf(PageId),
    Internal(PageId, &'a InternalNode<'a, K, &'a [u8]>, Option<usize>),
}

fn search<F, K>(key: &K, tx: &WriteTransaction, mut f: F)
where
    F: FnMut(Step<K>),
    K: FixedSize,
{
    let mut current = tx.root();

    loop {
        let page = tx.get_page(current).unwrap();

        let found_leaf = page.as_node(|node: Node<K, &[u8]>| {
            if let Some(inode) = node.try_as_internal() {
                let upper_pivot = match inode.keys().binary_search(key) {
                    Ok(pos) => Some(pos + 1),
                    Err(pos) => Some(pos),
                }
                .filter(|pos| pos < &inode.children().len());

                f(Step::Internal(page.id(), &inode, upper_pivot));

                if let Some(upper_pivot) = upper_pivot {
                    current = inode.children().get(upper_pivot);
                } else {
                    let last = inode.children().len().checked_sub(1).unwrap();
                    current = inode.children().get(last);
                }
                false
            } else {
                f(Step::Leaf(page.id()));
                true
            }
        });

        if found_leaf {
            return;
        }
    }
}

impl<'txbuilder, 'txmanager: 'txbuilder, 'storage: 'txmanager, K>
    DeleteBacktrack<'txbuilder, 'txmanager, 'storage, K>
where
    K: FixedSize,
{
    pub(crate) fn new_search_for(
        tx: &'txbuilder mut WriteTransaction<'txmanager, 'storage>,
        key: &K,
    ) -> Self {
        let mut backtrack = vec![];
        let mut parent_info = vec![];

        search(key, tx, |step| match step {
            Step::Leaf(page_id) => backtrack.push(page_id),
            Step::Internal(page_id, inode, upper_pivot) => {
                backtrack.push(page_id);
                let anchor = upper_pivot
                    .or_else(|| inode.keys().len().checked_sub(1))
                    .and_then(|up| up.checked_sub(1));

                let left_sibling_id = anchor.and_then(|pos| inode.children().try_get(pos));

                let right_sibling_id = anchor
                    .map(|pos| pos + 2)
                    .or(Some(1))
                    .and_then(|pos| inode.children().try_get(pos));

                parent_info.push((anchor, left_sibling_id, right_sibling_id));
            }
        });

        DeleteBacktrack {
            tx,
            backtrack,
            parent_info,
            new_root: None,
            phantom_key: PhantomData,
        }
    }

    pub fn get_next<'this>(
        &'this mut self,
    ) -> Result<Option<DeleteNextElement<'this, 'txbuilder, 'txmanager, 'storage, K>>, std::io::Error>
    {
        let id = match self.backtrack.pop() {
            Some(id) => id,
            None => return Ok(None),
        };

        if self.backtrack.is_empty() {
            assert!(self.new_root.is_none());
            self.new_root = Some(id);
        }

        let parent_info = match self.backtrack.last() {
            Some(parent) => {
                // we need the parent id, which is the next node in the stack, but we should not pop, because it would be the next node to process
                let (anchor, left, right) = self.parent_info.pop().expect("missing parent info");
                Some((parent, anchor, left, right))
            }
            None => None,
        };

        let next = match self.tx.mut_page(id)? {
            transaction::MutablePage::NeedsParentRedirect(rename_in_parents) => {
                // recursively clone(if they are not already used for some operation in the same transaction)
                // and redirect the whole path to this node.
                // Here redirect means clone the nodes and point the parents to the clone of its child
                let mut rename_in_parents = Some(rename_in_parents);
                let mut finished = None;
                for id in self.backtrack.iter().rev() {
                    let result = rename_in_parents
                        .take()
                        .unwrap()
                        .redirect_parent_pointer::<K>(*id)?;

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
                    // None means we got to the root of the tree
                    None => rename_in_parents.unwrap().finish(),
                }
            }
            transaction::MutablePage::InTransaction(handle) => handle,
        };

        let mut_context = match parent_info {
            Some((parent, _anchor, left_id, right_id)) => {
                let parent = match self.tx.mut_page(*parent)? {
                    MutablePage::InTransaction(handle) => handle,
                    _ => unreachable!(),
                };
                Some(MutableContext {
                    parent,
                    // anchor is an index into the keys array of a node used to find the current node in the parent without searching. The leftmost(lowest) child has None as anchor
                    // this means it's inmediate right sibling would have anchor of 0, and so on.
                    left_id,
                    right_id,
                    current_id: id,
                    backtrack: self,
                })
            }
            None => None,
        };

        let (left, right) = match parent_info {
            Some((_parent, _anchor, left, right)) => {
                let left = left.and_then(|id| self.tx.get_page(id));
                let right = right.and_then(|id| self.tx.get_page(id));

                (left, right)
            }
            None => (None, None),
        };

        let anchor = parent_info.and_then(|(_, anchor, _, _)| anchor);
        let next_element = NextElement {
            next,
            anchor,
            left,
            right,
            backtrack: self,
        };

        Ok(Some(DeleteNextElement {
            next_element,
            mut_context,
        }))
    }

    pub fn delete_node(&self, page_id: PageId) {
        self.tx.delete_node(page_id)
    }
}

impl<'txbuilder, 'txmanager: 'txbuilder, 'index: 'txmanager, K>
    InsertBacktrack<'txbuilder, 'txmanager, 'index, K>
where
    K: FixedSize,
{
    pub(crate) fn new_search_for(
        tx: &'txbuilder mut WriteTransaction<'txmanager, 'index>,
        key: &K,
    ) -> Self {
        let mut backtrack = vec![];
        search(key, tx, |step| match step {
            Step::Leaf(page_id) => backtrack.push(page_id),
            Step::Internal(page_id, _, _) => backtrack.push(page_id),
        });

        InsertBacktrack {
            tx,
            backtrack,
            new_root: None,
            phantom_key: PhantomData,
        }
    }

    pub fn get_next(&mut self) -> Result<Option<PageRefMut<'_>>, std::io::Error> {
        let id = match self.backtrack.pop() {
            Some(id) => id,
            None => return Ok(None),
        };

        if self.backtrack.is_empty() {
            assert!(self.new_root.is_none());
            self.new_root = Some(id);
        }

        match self.tx.mut_page(id)? {
            transaction::MutablePage::NeedsParentRedirect(rename_in_parents) => {
                // this part may be tricky, we need to recursively clone and redirect all the path
                // from the root to the node we are writing to. We need the backtrack stack, because
                // that's the only way to get the parent of a node (because there are no parent pointers)
                // so we iterate it in reverse but without consuming the stack (as we still need it for the
                // rest of the insertion algorithm)
                let mut rename_in_parents = rename_in_parents;
                for id in self.backtrack.iter().rev() {
                    let result = rename_in_parents.redirect_parent_pointer::<K>(*id)?;

                    match result {
                        MutablePage::NeedsParentRedirect(rename) => rename_in_parents = rename,
                        MutablePage::InTransaction(handle) => return Ok(Some(handle)),
                    }
                }
                Ok(Some(rename_in_parents.finish()))
            }
            transaction::MutablePage::InTransaction(handle) => Ok(Some(handle)),
        }
    }

    pub fn has_next(&self) -> bool {
        self.backtrack.last().is_some()
    }

    pub fn add_new_node(&mut self, mem_page: MemPage) -> Result<PageId, std::io::Error> {
        self.tx.add_new_node(mem_page)
    }

    pub fn new_root(&mut self, mem_page: MemPage) -> Result<(), std::io::Error> {
        let id = self.tx.add_new_node(mem_page)?;
        self.new_root = Some(id);

        Ok(())
    }
}

impl<'txbuilder, 'txmanager: 'txbuilder, 'index: 'txmanager, K>
    UpdateBacktrack<'txbuilder, 'txmanager, 'index, K>
where
    K: FixedSize,
{
    pub(crate) fn new_search_for(
        tx: &'txbuilder mut WriteTransaction<'txmanager, 'index>,
        key: &K,
    ) -> Self {
        let mut backtrack = vec![];
        search(key, tx, |step| match step {
            Step::Leaf(page_id) => backtrack.push(page_id),
            Step::Internal(page_id, _, _) => backtrack.push(page_id),
        });

        UpdateBacktrack {
            tx,
            backtrack,
            key_to_update: key.clone(),
            new_root: None,
            phantom_key: PhantomData,
        }
    }

    pub fn update<V: FixedSize>(&mut self, new_value: V) -> Result<(), std::io::Error> {
        let leaf = match self.backtrack.pop() {
            Some(id) => id,
            None => return Ok(()),
        };

        let position_to_update =
            match self
                .tx
                .get_page(leaf)
                .unwrap()
                .as_node(|node: Node<K, &[u8]>| {
                    node.as_leaf::<V>()
                        .keys()
                        .binary_search(&self.key_to_update)
                }) {
                Ok(pos) => pos,
                Err(_) => return Ok(()),
            };

        let mut page_handle = match self.tx.mut_page(leaf)? {
            transaction::MutablePage::NeedsParentRedirect(rename_in_parents) => {
                let mut rename_in_parents = Some(rename_in_parents);
                let handle = loop {
                    let id = match self.backtrack.pop() {
                        Some(id) => id,
                        None => {
                            break None;
                        }
                    };

                    if self.backtrack.is_empty() {
                        self.new_root = Some(id);
                    }

                    let result = rename_in_parents
                        .take()
                        .unwrap()
                        .redirect_parent_pointer::<K>(id)?;

                    match result {
                        MutablePage::NeedsParentRedirect(rename) => {
                            rename_in_parents = Some(rename)
                        }
                        MutablePage::InTransaction(handle) => break Some(handle),
                    };
                };

                handle.unwrap_or_else(|| rename_in_parents.take().unwrap().finish())
            }
            transaction::MutablePage::InTransaction(handle) => handle,
        };

        page_handle.as_node_mut(|mut node: Node<K, &mut [u8]>| {
            node.as_leaf_mut()
                .values_mut()
                .update(position_to_update, &new_value)
                .expect("position to update was not in range")
        });

        Ok(())
    }
}

impl<'txbuilder, 'txmanager: 'txbuilder, 'storage: 'txmanager, K> Drop
    for InsertBacktrack<'txbuilder, 'txmanager, 'storage, K>
where
    K: FixedSize,
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
    K: FixedSize,
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
    for UpdateBacktrack<'txbuilder, 'txmanager, 'storage, K>
where
    K: FixedSize,
{
    fn drop(&mut self) {
        if let Some(new_root) = self.new_root {
            self.tx.current_root.set(new_root);
        } else {
            self.tx.current_root.set(*self.backtrack.first().unwrap());
        }
    }
}
