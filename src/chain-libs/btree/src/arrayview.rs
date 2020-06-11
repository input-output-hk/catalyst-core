// Replace the things in this module with a dependency (but still don't know which one)
// Otherwise, there is unsafe code here that needs to be properly guarded with asserts

use std::borrow::Borrow;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Range;

use byteorder::{ByteOrder as _, LittleEndian};

use crate::Storeable;

pub(crate) struct ArrayView<'elements, T: 'elements, E> {
    pub data: T,
    pub len: usize,
    element_size: Size,
    pub phantom: PhantomData<&'elements [E]>,
}

enum Size {
    Static(usize),
    Dynamic(usize),
}

impl<'elements, E, T> ArrayView<'elements, T, E>
where
    E: Clone + Debug + Ord + for<'a> Storeable<'a> + Sized,
    T: AsRef<[u8]> + 'elements,
{
    pub(crate) fn new_static_size(data: T, len: usize) -> ArrayView<'elements, T, E> {
        assert_eq!(
            data.as_ref()
                .as_ptr()
                .align_offset(std::mem::align_of::<E>()),
            0
        );

        ArrayView {
            data,
            len,
            element_size: Size::Static(std::mem::size_of::<E>()),
            phantom: PhantomData,
        }
    }
}

impl<'elements, E, T> ArrayView<'elements, T, E>
where
    E: Clone + Debug + Ord + for<'a> Storeable<'a>,
    T: AsRef<[u8]> + 'elements,
{
    pub(crate) fn new_dynamic_size(
        data: T,
        len: usize,
        element_size: usize,
    ) -> ArrayView<'elements, T, E> {
        assert_eq!(
            if element_size < 8 {
                data.as_ref().as_ptr().align_offset(element_size)
            } else {
                0
            },
            0
        );

        ArrayView {
            data,
            len,
            element_size: Size::Dynamic(element_size),
            phantom: PhantomData,
        }
    }

    // TODO: add a marker type so this only can be used on sorted views?
    pub(crate) fn binary_search<'me, Q: 'me>(&'me self, element: &Q) -> Result<usize, usize>
    where
        Q: Ord,
        E: Borrow<Q>,
    {
        let stride = usize::from(&self.element_size);
        let data: &'me [u8] = self.data.as_ref();

        // this is O(N) in operations, but the search is O(log(N)) in comparissons
        let wrapper: Vec<&'me [u8]> = data[0..self.len() * stride].chunks_exact(stride).collect();

        let de: Vec<<E as Storeable>::Output> = wrapper
            .iter()
            .map(|slice| E::read(&slice[..]).unwrap())
            .collect();

        de.binary_search_by_key(&element.borrow(), |s| s.borrow().borrow())
    }

    pub(crate) fn linear_search<'me, Q: 'me>(&'me self, element: Q) -> Option<usize>
    where
        Q: Borrow<E> + Eq + PartialEq,
    {
        let stride = usize::from(&self.element_size);
        let data: &'me [u8] = self.data.as_ref();

        let wrapper: Vec<&'me [u8]> = data[0..self.len() * stride].chunks_exact(stride).collect();

        let de: Vec<<E as Storeable>::Output> = wrapper
            .iter()
            .map(|slice| E::read(&slice[..]).unwrap())
            .collect();

        de.iter()
            .enumerate()
            .find(|(_i, s)| element.borrow() == (*s).borrow())
            .map(|(i, _)| i)
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn get(&self, pos: usize) -> <E as Storeable<'_>>::Output {
        self.try_get(pos).unwrap()
    }

    pub(crate) fn try_get(&self, pos: usize) -> Option<<E as Storeable<'_>>::Output> {
        if pos < self.len() {
            Some(
                E::read(
                    &self.data.as_ref()[pos * usize::from(&self.element_size)
                        ..(pos + 1) * usize::from(&self.element_size)],
                )
                .expect("Couldn't deserialize key"),
            )
        } else {
            None
        }
    }

    pub(crate) fn sub<'a: 'elements>(
        &'a self,
        range: Range<usize>,
    ) -> ArrayView<&'elements [u8], E> {
        match self.element_size {
            Size::Static(n) => ArrayView::new_dynamic_size(
                &self.data.as_ref()[range.start * usize::from(&self.element_size)
                    ..range.end * usize::from(&self.element_size)],
                range.end.checked_sub(range.start).unwrap(),
                n,
            ),
            Size::Dynamic(n) => ArrayView::new_dynamic_size(
                &self.data.as_ref()[range.start * usize::from(&self.element_size)
                    ..range.end * usize::from(&self.element_size)],
                range.end.checked_sub(range.start).unwrap(),
                n,
            ),
        }
    }

    pub(crate) fn iter(&self) -> ArrayViewIter<T, E> {
        ArrayViewIter { view: self, pos: 0 }
    }
}

impl<'elements, E, T> ArrayView<'elements, T, E>
where
    E: Clone + Debug + Ord + for<'a> Storeable<'a>,
    T: AsRef<[u8]> + AsMut<[u8]> + 'elements,
{
    pub(crate) fn insert(&mut self, pos: usize, element: &E) -> Result<(), ()> {
        if self.len() < self.data.as_ref().len() / usize::from(&self.element_size) {
            unsafe {
                let src: *mut u8 = self
                    .data
                    .as_mut()
                    .as_mut_ptr()
                    .add(pos * usize::from(&self.element_size));
                let dst: *mut u8 = self
                    .data
                    .as_mut()
                    .as_mut_ptr()
                    .add((pos + 1) * usize::from(&self.element_size));

                std::ptr::copy(
                    src,
                    dst,
                    self.len().checked_sub(pos).unwrap() * usize::from(&self.element_size),
                );

                let mut buffer = vec![0u8; usize::from(&self.element_size)];

                element
                    .write(&mut buffer[..])
                    .expect("Couldn't serialize key");

                std::ptr::copy(
                    buffer.as_ptr() as *const u8,
                    src,
                    usize::from(&self.element_size),
                );

                self.len += 1;

                Ok(())
            }
        } else {
            Err(())
        }
    }

    pub(crate) fn delete(&mut self, pos: usize) -> Result<(), ()> {
        if pos < self.len() {
            unsafe {
                let src: *mut u8 = self
                    .data
                    .as_mut()
                    .as_mut_ptr()
                    .add((pos + 1) * usize::from(&self.element_size));
                let dst: *mut u8 = self
                    .data
                    .as_mut()
                    .as_mut_ptr()
                    .add(pos * usize::from(&self.element_size));

                std::ptr::copy(
                    src,
                    dst,
                    self.len()
                        .checked_sub(1)
                        .and_then(|n| n.checked_sub(pos))
                        .unwrap()
                        * usize::from(&self.element_size),
                );

                self.len -= 1;

                Ok(())
            }
        } else {
            Err(())
        }
    }

    pub(crate) fn update(&mut self, pos: usize, key: &E) -> Result<(), ()> {
        if pos < self.len() {
            unsafe {
                let dst: *mut u8 = self
                    .data
                    .as_mut()
                    .as_mut_ptr()
                    .add(pos * usize::from(&self.element_size));

                let mut buffer = vec![0u8; usize::from(&self.element_size)];
                key.write(&mut buffer[..]).expect("Couldn't serialize key");

                std::ptr::copy(
                    buffer.as_ptr() as *const u8,
                    dst,
                    usize::from(&self.element_size),
                );

                Ok(())
            }
        } else {
            Err(())
        }
    }

    pub fn append(&mut self, element: &E) -> Result<(), ()> {
        let current_len = self.len();
        self.insert(current_len, element)
    }
}

pub(crate) struct ArrayViewIter<'elements, T, E> {
    view: &'elements ArrayView<'elements, T, E>,
    pos: usize,
}

impl<'elements, T, E> Iterator for ArrayViewIter<'elements, T, E>
where
    E: Clone + std::fmt::Debug + Ord + for<'a> Storeable<'a>,
    T: AsRef<[u8]>,
{
    type Item = <E as Storeable<'elements>>::Output;
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < self.view.len() {
            let e = self.view.try_get(self.pos);
            self.pos += 1;
            e
        } else {
            None
        }
    }
}

impl From<&Size> for usize {
    fn from(s: &Size) -> usize {
        match s {
            Size::Static(n) => *n,
            Size::Dynamic(n) => *n,
        }
    }
}

impl<'a> Storeable<'a> for u32 {
    type Error = std::io::Error;
    type Output = Self;

    fn write(&self, buf: &mut [u8]) -> Result<(), Self::Error> {
        LittleEndian::write_u32(buf, *self);
        Ok(())
    }

    fn read(buf: &'a [u8]) -> Result<Self::Output, Self::Error> {
        Ok(LittleEndian::read_u32(buf))
    }
}

impl<'a> Storeable<'a> for u64 {
    type Error = std::io::Error;
    type Output = Self;

    fn write(&self, buf: &mut [u8]) -> Result<(), Self::Error> {
        LittleEndian::write_u64(buf, *self);
        Ok(())
    }

    fn read(buf: &'a [u8]) -> Result<Self::Output, Self::Error> {
        Ok(LittleEndian::read_u64(buf))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn insert_and_get_are_compatible() {
        let numbers = vec![0u32, 1, 2, 3, 4, 5, 7];
        let key_size = std::mem::size_of::<u32>();
        let mut buffer = vec![0u8; numbers.len() * key_size];
        let mut view = ArrayView::<&mut [u8], u32>::new_static_size(&mut buffer, 0);

        for (i, n) in numbers.iter().enumerate() {
            view.insert(i, n).unwrap();
        }

        assert_eq!(view.len(), numbers.len());
        for (n, v) in numbers.iter().zip(view.iter()) {
            assert_eq!(n, v.borrow())
        }

        assert_eq!(view.binary_search(&0).unwrap(), 0);
        assert_eq!(view.binary_search(&1).unwrap(), 1);
        assert_eq!(view.binary_search(&7).unwrap(), 6);
    }
    #[test]
    fn can_delete_in_the_middle() {
        let numbers = vec![0u32, 1, 2];
        let key_size = std::mem::size_of::<u32>();
        let mut buffer = vec![0u8; numbers.len() * key_size];
        let mut view = ArrayView::<&mut [u8], u32>::new_static_size(&mut buffer, 0);

        for (i, n) in numbers.iter().enumerate() {
            view.insert(i, n).unwrap();
        }

        view.delete(1usize).unwrap();

        assert_eq!(view.len(), numbers.len() - 1);
        let result: Vec<u32> = view.iter().collect();
        let expected = vec![0u32, 2];
        assert_eq!(result, expected);
    }
    #[test]
    #[should_panic]
    fn wrong_alignment_is_detected() {
        let buffer = vec![0u32; 8];
        let unaligning_offset = 1usize;
        let arbitrary_slice_len = 4usize;
        let unaligned_slice: &[u8] = unsafe {
            std::slice::from_raw_parts(
                buffer.as_ptr().cast::<u8>().add(unaligning_offset),
                arbitrary_slice_len,
            )
        };

        let arbitrary_view_len = 0usize;
        ArrayView::<&[u8], u32>::new_static_size(unaligned_slice, arbitrary_view_len);
    }

    #[test]
    fn can_sub() {
        let numbers = vec![0u32, 1, 2, 3, 4, 5, 7];
        let mut buffer = vec![0u8; numbers.len() * size_of::<u32>()];
        let mut view = ArrayView::<&mut [u8], u32>::new_static_size(&mut buffer, 0);

        for (i, n) in numbers.iter().enumerate() {
            view.insert(i, n).unwrap();
        }

        assert_eq!(view.len(), numbers.len());
        for (n, v) in numbers.iter().take(3).zip(view.sub(0..3).iter()) {
            assert_eq!(n, v.borrow())
        }
    }

    #[test]
    fn can_insert_in_the_middle() {
        let view_size = 4;
        let alloc = unsafe {
            std::alloc::alloc(
                std::alloc::Layout::from_size_align(size_of::<u32>() * view_size, 8).unwrap(),
            )
        };
        let slice = unsafe { std::slice::from_raw_parts_mut(alloc, view_size * size_of::<u32>()) };
        let mut view = ArrayView::<&mut [u8], u32>::new_static_size(slice, 0);
        view.insert(0, &0u32).unwrap();
        view.insert(1, &2u32).unwrap();
        view.insert(1, &1u32).unwrap();

        assert_eq!(view.len(), 3);
        let result: Vec<u32> = view.iter().collect();
        let expected = vec![0u32, 1, 2];
        assert_eq!(result, expected);
    }
}
