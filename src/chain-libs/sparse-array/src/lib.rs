///! Implementation of a sparse array storing maximum of 256 elements

mod bitmap;
mod fast;
mod sparse_array;

#[cfg(test)]
mod testing;

pub use crate::{
    fast::{FastSparseArray, FastSparseArrayBuilder, FastSparseArrayIter},
    sparse_array::{SparseArray, SparseArrayBuilder, SparseArrayIter},
};
