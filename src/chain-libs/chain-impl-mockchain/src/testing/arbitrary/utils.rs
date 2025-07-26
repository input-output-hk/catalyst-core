use quickcheck::{Arbitrary, Gen, TestResult};
use std::collections::{hash_map, HashMap, HashSet};
use std::fmt::Debug;
use std::{
    cmp::{self, Eq, PartialEq},
    hash::Hash,
};

pub fn factorize<G: Gen>(number: u32, gen: &mut G) -> Vec<u32> {
    let mut current_remainder = number;
    let mut output = Vec::new();

    while current_remainder > 0 {
        let part = cmp::max(u32::arbitrary(gen) % current_remainder, 1);
        output.push(part);
        current_remainder -= part;
    }
    output
}

pub fn split_vec<G: Gen, T>(source: &[T], gen: &mut G, number_of_splits: usize) -> Vec<Vec<T>>
where
    T: std::clone::Clone,
{
    let mut matrix: Vec<Vec<T>> = (0..number_of_splits)
        .map(|_| Vec::with_capacity(number_of_splits))
        .collect();
    for x in source {
        let index = usize::arbitrary(gen) % number_of_splits;
        matrix.get_mut(index).unwrap().push(x.clone());
    }
    matrix
}

pub fn choose_random_vec_subset<G: Gen, T>(source: &[T], gen: &mut G) -> Vec<T>
where
    T: std::clone::Clone,
{
    let arbitrary_indexes = choose_random_indexes(gen, source.len());
    // create sub collecion from arbitrary indexes
    source
        .iter()
        .cloned()
        .enumerate()
        .filter(|(i, _)| arbitrary_indexes.contains(i))
        .map(|(_, e)| e)
        .collect()
}

pub fn choose_random_set_subset<G: Gen, T>(
    source: &HashSet<T, hash_map::RandomState>,
    gen: &mut G,
) -> HashSet<T>
where
    T: std::clone::Clone + Eq + Hash,
{
    let arbitrary_indexes = choose_random_indexes(gen, source.len());
    // create sub collecion from arbitrary indexes
    source
        .iter()
        .cloned()
        .enumerate()
        .filter(|(i, _)| arbitrary_indexes.contains(i))
        .map(|(_, e)| e)
        .collect()
}

pub fn choose_random_item<G: Gen, T>(source: &[T], gen: &mut G) -> T
where
    T: std::clone::Clone,
{
    let index = usize::arbitrary(gen) % source.len();
    source.get(index).cloned().unwrap()
}

pub fn choose_random_map_subset<G: Gen, T, U>(
    source: &HashMap<T, U, hash_map::RandomState>,
    gen: &mut G,
) -> HashMap<T, U>
where
    T: Clone + PartialEq + Eq + Hash,
    U: std::clone::Clone,
{
    let keys: Vec<T> = source.keys().cloned().collect();
    let randomized_key = choose_random_vec_subset(&keys, gen);
    randomized_key
        .iter()
        .cloned()
        .map(|x| (x.clone(), source.get(&x).unwrap().clone()))
        .collect()
}

pub fn choose_random_indexes<G: Gen>(gen: &mut G, upper_bound: usize) -> HashSet<usize> {
    let lower_bound = 1;
    let mut arbitrary_indexes = HashSet::new();

    // set limit between lower_bound and upper_bound
    let random_length = cmp::max(usize::arbitrary(gen) % upper_bound, lower_bound);

    // choose arbitrary non-repertive indexes
    while arbitrary_indexes.len() < random_length {
        let random_number = usize::arbitrary(gen) % upper_bound;
        arbitrary_indexes.insert(random_number);
    }
    arbitrary_indexes
}

/// Struct helps gather all verifications and then decide if test if failed or not. Currently it's tightly coupled
/// with quickcheck crate
#[derive(Default)]
pub struct Verify(Vec<TestResult>);

impl Verify {
    pub fn new() -> Self {
        Verify(Vec::new())
    }

    pub fn verify_eq<A: PartialEq + Debug>(&mut self, expected: A, actual: A, desc: &str) {
        let result = if expected == actual {
            TestResult::error(format!(
                "expected {} {:?}, but got {:?}",
                desc, expected, actual
            ))
        } else {
            TestResult::passed()
        };
        self.0.push(result);
    }

    pub fn verify(&mut self, is_true: bool, desc: &str) {
        let result = if is_true {
            TestResult::error(format!("expected {} to be true, but got false", desc))
        } else {
            TestResult::passed()
        };
        self.0.push(result);
    }

    pub fn get_result(&self) -> TestResult {
        self.0
            .iter()
            .find(|&x| TestResult::is_failure(x))
            .cloned()
            .unwrap_or_else(TestResult::passed)
    }
}
