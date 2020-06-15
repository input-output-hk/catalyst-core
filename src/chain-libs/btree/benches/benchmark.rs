use btree::{BTreeStore, FixedSize, Storeable};
use criterion::{criterion_group, criterion_main, Criterion};
extern crate rand;
use crate::rand::rngs::StdRng;
use crate::rand::Rng as _;
use crate::rand::SeedableRng;
use byteorder::{ByteOrder, LittleEndian};
use std::convert::TryInto;

static SEED: u64 = 11;

const BLOB_SIZE: usize = 1024 * 10; // 10 kb?

#[derive(Debug, Clone, Ord, Eq, PartialEq, PartialOrd)]
pub struct U64Key(pub u64);

impl<'a> Storeable<'a> for U64Key {
    type Error = std::io::Error;
    type Output = Self;

    fn write(&self, buf: &mut [u8]) -> Result<(), Self::Error> {
        LittleEndian::write_u64(buf, self.0);
        Ok(())
    }
    fn read(buf: &'a [u8]) -> Result<Self::Output, Self::Error> {
        Ok(U64Key(LittleEndian::read_u64(buf)))
    }
}

impl FixedSize for U64Key {
    fn max_size() -> usize {
        std::mem::size_of::<Self>()
    }
}

fn random_blob(rng: &mut impl rand::Rng) -> Box<[u8]> {
    let mut buf = vec![0u8; BLOB_SIZE];
    rng.fill(&mut buf[..]);
    buf.into_boxed_slice()
}

fn single_key_insertion(c: &mut Criterion) {
    // TODO: Maybe create a temp file somehow?
    let dir_path = "benchmark_single_key_insertion";
    let key_size = std::mem::size_of::<U64Key>();
    let page_size = 4096;

    let tree: BTreeStore<U64Key> =
        BTreeStore::new(dir_path, key_size.try_into().unwrap(), page_size).unwrap();

    let n: u64 = 200_000;

    let mut rng = StdRng::seed_from_u64(SEED);

    tree.insert_many(
        (0..n)
            .step_by(2)
            .map(|i| (U64Key(i), random_blob(&mut rng))),
    )
    .expect("Couldn't insert setup values");

    c.bench_function("insertion", |b| {
        b.iter(|| {
            let r: u64 = rng.gen();
            let key = if r % 2 == 0 { r + 1 } else { r };

            let blob_to_insert: Box<[u8]> = random_blob(&mut rng);
            tree.insert(U64Key(key), &blob_to_insert[..]).unwrap_or(());

            assert_eq!(
                tree.get(&U64Key(key)).unwrap().expect("Key not found"),
                &blob_to_insert[..]
            );
        })
    });

    std::fs::remove_dir_all(dir_path).unwrap();
}

fn single_key_search(c: &mut Criterion) {
    let dir_path = "benchmark_single_key_search";
    let key_size = std::mem::size_of::<U64Key>();
    let page_size = 4096;

    let tree: BTreeStore<U64Key> =
        BTreeStore::new(dir_path, key_size.try_into().unwrap(), page_size).unwrap();

    let n: u64 = 200_000;

    let mut rng = StdRng::seed_from_u64(SEED);

    use std::collections::HashMap;
    let mut data = HashMap::new();

    for i in 0u64..n {
        data.insert(i, random_blob(&mut rng));
    }

    tree.insert_many(data.iter().map(|(key, value)| (U64Key(*key), value)))
        .expect("Couldn't insert setup values");

    c.bench_function("search", |b| {
        b.iter(|| {
            let key: u64 = rng.gen_range(0, n);
            assert_eq!(
                tree.get(&U64Key(key)).unwrap().expect("Key not found"),
                &data.get(&key).unwrap()[..]
            )
        })
    });

    std::fs::remove_dir_all(dir_path).unwrap();
}

criterion_group!(
    name = insertion;
    config = Criterion::default().sample_size(10);
    targets = single_key_insertion
);

criterion_group!(
    name = search;
    config = Criterion::default().sample_size(500);
    targets = single_key_search
);

criterion_main!(insertion, search);
