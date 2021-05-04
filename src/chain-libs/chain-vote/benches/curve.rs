//! Benchmarks to compare the different curves, sec2 and curve25519 (with ristretto prime order
//! group), instantiated with default flag and ristretto225 respectively.

use chain_vote::debug::gang;
use chain_vote::debug::gang::GroupElement;
use criterion::{criterion_group, criterion_main, Criterion};
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;

fn mul(c: &mut Criterion) {
    let mut rng = ChaCha20Rng::from_seed([0u8; 32]);
    let g = gang::GroupElement::from_hash(b"random element");
    let scalar = gang::Scalar::random(&mut rng);

    c.bench_function("Point - Scalar multiplication", |b| b.iter(|| &g * &scalar));
}

fn addition(c: &mut Criterion) {
    let point1 = gang::GroupElement::from_hash(b"random element");
    let point2 = gang::GroupElement::from_hash(b"random element");

    c.bench_function("Group element addition", |b| b.iter(|| &point1 + &point2));
}

fn from_hash(c: &mut Criterion) {
    let string = [0u8; 32];

    c.bench_function("Group element from hash", |b| {
        b.iter(|| GroupElement::from_hash(&string))
    });
}

fn to_bytes(c: &mut Criterion) {
    let point = GroupElement::from_hash(b"random element");

    c.bench_function("Group element compression", |b| b.iter(|| point.to_bytes()));
}

fn decompress(c: &mut Criterion) {
    let point = GroupElement::from_hash(b"random element");
    let bytes = point.to_bytes();

    c.bench_function("Group element decompression", |b| {
        b.iter(|| GroupElement::from_bytes(&bytes))
    });
}

fn scalar_multiplication(c: &mut Criterion) {
    let mut rng = ChaCha20Rng::from_seed([0u8; 32]);
    let scalar1 = gang::Scalar::random(&mut rng);
    let scalar2 = gang::Scalar::random(&mut rng);

    c.bench_function("Scalar multiplication", |b| b.iter(|| &scalar1 * &scalar2));
}

fn scalar_power(c: &mut Criterion) {
    let mut rng = ChaCha20Rng::from_seed([0u8; 32]);
    let scalar = gang::Scalar::random(&mut rng);

    c.bench_function("Scalar exponentiation", |b| {
        b.iter(|| scalar.power(191328475619823764usize))
    });
}

fn scalar_inversion(c: &mut Criterion) {
    let mut rng = ChaCha20Rng::from_seed([0u8; 32]);
    let scalar = gang::Scalar::random(&mut rng);

    c.bench_function("Scalar inversion", |b| b.iter(|| scalar.inverse()));
}

criterion_group!(
    name = group_ops;
    config = Criterion::default();
    targets =
    mul,
    from_hash,
    to_bytes,
    decompress,
    addition,
    scalar_multiplication,
    scalar_power,
    scalar_inversion,
);
criterion_main!(group_ops);
