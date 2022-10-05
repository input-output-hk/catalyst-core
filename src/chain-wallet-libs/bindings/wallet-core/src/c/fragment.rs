use crate::{Error, Result};
use chain_core::{packer::Codec, property::DeserializeFromSlice};
use chain_impl_mockchain::fragment::Fragment;
use core::slice;

use super::{FragmentPtr, NulPtr, FRAGMENT_ID_LENGTH};

/// # Safety
///
/// buffer must be non null and point to buffer_length bytes of valid memory.
///
pub unsafe fn fragment_from_raw(
    buffer: *const u8,
    buffer_length: usize,
    fragment_out: *mut FragmentPtr,
) -> Result {
    if buffer.is_null() {
        return Error::invalid_input("buffer").with(NulPtr).into();
    }

    let fragment_out_ref = non_null_mut!(fragment_out);

    let bytes = slice::from_raw_parts(buffer, buffer_length);

    let fragment = match Fragment::deserialize_from_slice(&mut Codec::new(bytes)) {
        Ok(fragment) => fragment,
        Err(_e) => return Error::invalid_fragment().into(),
    };

    let fragment = Box::new(fragment);

    *fragment_out_ref = Box::into_raw(fragment);

    Result::success()
}

/// # Safety
///
/// fragment_ptr must be a pointer to memory allocated by this library, for
/// example, with `fragment_from_raw`
/// id_out must point to FRAGMENT_ID_LENGTH bytes of valid allocated writable
/// memory
/// This function checks for null pointers
///
pub unsafe fn fragment_id(fragment_ptr: FragmentPtr, id_out: *mut u8) -> Result {
    let fragment = non_null!(fragment_ptr);

    let id = fragment.hash();

    let bytes = id.as_bytes();

    assert_eq!(bytes.len(), FRAGMENT_ID_LENGTH);

    std::ptr::copy(bytes.as_ptr(), id_out, bytes.len());

    Result::success()
}

/// # Safety
///
/// This function checks for null pointers, but take care that fragment_ptr was
/// previously allocated by this library for example with fragment_from_raw
///
pub unsafe fn fragment_delete(fragment_ptr: FragmentPtr) {
    if !fragment_ptr.is_null() {
        Box::from_raw(fragment_ptr as FragmentPtr);
    }
}
