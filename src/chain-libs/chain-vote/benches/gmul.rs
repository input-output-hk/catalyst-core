use chain_vote::debug::gang;
use criterion::{criterion_group, criterion_main, Criterion};

fn mul(c: &mut Criterion) {
    let g = gang::GroupElement::generator() * gang::Scalar::from_u64(100);
    c.bench_function("mul", |b| b.iter(|| &g + &g));
}

criterion_group!(gmul, mul);
criterion_main!(gmul);
