use std::alloc;

const MEM_ALIGNMENT: usize = 8;

/// Box-like structure but with custom alignment
/// all nodes are allocated on this structure, but ideally at some point we could just use pointers into the mmap
#[derive(Debug)]
pub struct MemPage {
    data: *mut u8,
    layout: alloc::Layout,
}

impl Drop for MemPage {
    fn drop(&mut self) {
        unsafe { alloc::dealloc(self.data, self.layout) };
    }
}

impl MemPage {
    pub(crate) fn new(size: usize) -> MemPage {
        let layout =
            alloc::Layout::from_size_align(size, MEM_ALIGNMENT).expect("Memory layout error");
        let data = unsafe { alloc::alloc(layout) };
        if data.is_null() {
            panic!("Couldn't allocate memory");
        }
        MemPage { data, layout }
    }
}

impl AsMut<[u8]> for MemPage {
    fn as_mut(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.data as *mut u8, self.layout.size()) }
    }
}

impl AsRef<[u8]> for MemPage {
    fn as_ref(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.data as *const u8, self.layout.size()) }
    }
}

impl Clone for MemPage {
    fn clone(&self) -> MemPage {
        let mut new_alloc = MemPage::new(self.as_ref().len());
        new_alloc.as_mut().copy_from_slice(self.as_ref());
        new_alloc
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clone_does_not_alias() {
        // JUST IN CASE
        // definitely don't want aliasing here
        let first_page = MemPage::new(4096);
        let clone = first_page.clone();

        let first_pointer = first_page.as_ref().as_ptr();
        let clone_pointer = clone.as_ref().as_ptr();
        assert_ne!(first_pointer, clone_pointer);
        // XXX: assert distance is > size?
    }
}
