use criterion::{criterion_group, criterion_main, Criterion};

use imhamt::*;

use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeMap;

type Key = String;

const NB: usize = 1000;

fn keys() -> Vec<Key> {
    let mut v = Vec::with_capacity(NB);
    for i in 0..NB {
        v.push(format!("key {}", i))
    }
    v
}

fn bench_btreemap_insert(c: &mut Criterion) {
    c.bench_function("bench_btreemap_insert", |b| {
        b.iter(|| {
            let mut h: BTreeMap<Key, u32> = BTreeMap::new();
            for k in keys() {
                h.insert(k, 2);
            }
        });
    });
}

fn bench_hamt_insert(c: &mut Criterion) {
    c.bench_function("bench_hamt_insert", |b| {
        b.iter(|| {
            let mut h: Hamt<DefaultHasher, Key, u32> = Hamt::new();
            for k in keys() {
                h = h.insert(k, 2).unwrap()
            }
        })
    });
}

fn bench_btreemap_remove(c: &mut Criterion) {
    let mut h: BTreeMap<Key, u32> = BTreeMap::new();
    for k in keys() {
        h.insert(k, 2);
    }
    c.bench_function("bench_btreemap_remove", |b| {
        b.iter(|| {
            let mut h2 = h.clone();
            for k in keys() {
                h2.remove(&k);
            }
        })
    });
}

fn bench_hamt_remove(c: &mut Criterion) {
    let mut h: Hamt<DefaultHasher, Key, u32> = Hamt::new();
    for k in keys() {
        h = h.insert(k, 2).unwrap()
    }
    c.bench_function("bench_hamt_remove", |b| {
        b.iter(|| {
            let mut h2 = h.clone();
            for k in keys() {
                h2 = h2.remove_match(&k, &2).unwrap()
            }
        })
    });
}

criterion_group!(
    reference_btree,
    bench_btreemap_insert,
    bench_btreemap_remove,
);
criterion_group!(hamt, bench_hamt_insert, bench_hamt_remove);
criterion_main!(reference_btree, hamt);
