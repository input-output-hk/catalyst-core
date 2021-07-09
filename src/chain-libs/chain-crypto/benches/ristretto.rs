//! Benchmarks of the ristretto group over curve25519

use chain_crypto::ec::ristretto255::{GroupElement, Scalar};

use criterion::{criterion_group, criterion_main, Criterion};
use rand_core::OsRng;

fn mul(c: &mut Criterion) {
    let mut rng = OsRng;
    let g = GroupElement::from_hash(b"random element");
    let scalar = Scalar::random(&mut rng);

    c.bench_function("Point - Scalar multiplication", |b| b.iter(|| &g * &scalar));
}

fn addition(c: &mut Criterion) {
    let point1 = GroupElement::from_hash(b"random element");
    let point2 = GroupElement::from_hash(b"random element");

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
    let mut rng = OsRng;
    let scalar1 = Scalar::random(&mut rng);
    let scalar2 = Scalar::random(&mut rng);

    c.bench_function("Scalar multiplication", |b| b.iter(|| &scalar1 * &scalar2));
}

fn scalar_power(c: &mut Criterion) {
    let mut rng = OsRng;
    let scalar = Scalar::random(&mut rng);

    c.bench_function("Scalar exponentiation", |b| {
        b.iter(|| scalar.power(191328475619823764usize))
    });
}

fn scalar_inversion(c: &mut Criterion) {
    let mut rng = OsRng;
    let scalar = Scalar::random(&mut rng);

    c.bench_function("Scalar inversion", |b| b.iter(|| scalar.inverse()));
}

fn random_scalar(c: &mut Criterion) {
    let mut rng = OsRng;
    c.bench_function("Random scalar", |b| b.iter(|| Scalar::random(&mut rng)));
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
    random_scalar
);
criterion_main!(group_ops);
