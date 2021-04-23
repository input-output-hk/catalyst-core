use proptest::prelude::*;

const MAX_INDEX: usize = 255;

/// Generates a list of indexes for testing sparse arrays and bitmaps.
pub(crate) fn test_indexes() -> impl Strategy<Value = Vec<u8>> {
    // generate the list of free & occupied positions in a sparse aray
    prop::bits::bool_vec::between(0, MAX_INDEX).prop_map(|occupied_entries| {
        occupied_entries
            .into_iter()
            .enumerate()
            .filter(|(_, occupied)| *occupied) // take occupied positions...
            .map(|(i, _)| i as u8) // ...and get their indexes
            .collect()
    })
}

/// Generates an ordered list of entries for sparse arrays. There are no entries with repeating
/// numbers.
pub(crate) fn sparse_array_test_data() -> impl Strategy<Value = Vec<(u8, u8)>> {
    test_indexes().prop_flat_map(|indexes: Vec<u8>| {
        // populate an array with the given indexes
        prop::collection::vec(0..=u8::MAX, indexes.len()).prop_map(move |values| {
            let indexes = indexes.clone();
            indexes.into_iter().zip(values.into_iter()).collect()
        })
    })
}
