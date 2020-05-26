#[allow(clippy::module_inception)]
mod fragment;
mod id;

pub use fragment::Fragment;
pub use id::{try_ids_from_iter, FragmentId, FragmentIds};
