use chain_crypto::algorithms::sumed25519::{
    common::{Depth, Seed},
    sum::*,
};
use criterion::{criterion_group, criterion_main, Criterion};

fn keygen_with_depth(depth: Depth, c: &mut Criterion) {
    let seed = Seed::zero();
    c.bench_function("bench_btreemap_insert", |b| {
        b.iter(|| {
            let _ = keygen(depth, &seed);
        })
    });
}

fn update_with_depth(depth: Depth, nb_update: usize, c: &mut Criterion) {
    let seed = Seed::zero();
    let (sk_orig, _) = keygen(depth, &seed);
    c.bench_function("bench_btreemap_insert", |b| {
        b.iter(|| {
            let mut sk = sk_orig.clone();
            for _ in 0..(nb_update - 1) {
                update(&mut sk).unwrap()
            }
        })
    });
}

fn update_with_depth_skip(depth: Depth, nb_update_to_skip: usize, c: &mut Criterion) {
    let seed = Seed::zero();
    let (mut sk_orig, _) = keygen(depth, &seed);
    for _ in 0..(nb_update_to_skip - 1) {
        update(&mut sk_orig).unwrap()
    }
    c.bench_function("bench_btreemap_insert", |b| {
        b.iter(|| {
            let mut sk = sk_orig.clone();
            update(&mut sk).unwrap()
        })
    });
}

fn keygen_depth7(c: &mut Criterion) {
    keygen_with_depth(Depth(7), c)
}

fn keygen_depth8(c: &mut Criterion) {
    keygen_with_depth(Depth(8), c)
}

fn keygen_depth12(c: &mut Criterion) {
    keygen_with_depth(Depth(12), c)
}

fn sign_depth12(c: &mut Criterion) {
    let (sk, _) = keygen(Depth(12), &Seed::zero());
    let msg = [0u8; 256];
    c.bench_function("bench_btreemap_insert", |b| b.iter(|| sign(&sk, &msg)));
}

fn verify_depth12(c: &mut Criterion) {
    let (sk, pk) = keygen(Depth(12), &Seed::zero());
    let msg = [0u8; 256];
    let signature = sign(&sk, &msg);
    c.bench_function("bench_btreemap_insert", |b| {
        b.iter(|| verify(&pk, &msg, &signature))
    });
}

fn update2_depth2(c: &mut Criterion) {
    update_with_depth(Depth(2), 2, c)
}

fn update4_depth4(c: &mut Criterion) {
    update_with_depth(Depth(4), 4, c)
}

fn update16_depth8(c: &mut Criterion) {
    update_with_depth(Depth(8), 16, c)
}

fn update32_depth12(c: &mut Criterion) {
    update_with_depth(Depth(12), 32, c)
}

fn update128_depth16(c: &mut Criterion) {
    update_with_depth_skip(Depth(16), (1 << 16) - 1, c)
}

criterion_group!(bkeygen, keygen_depth7, keygen_depth8, keygen_depth12);
criterion_group!(bsign, sign_depth12);
criterion_group!(bverify, verify_depth12);
criterion_group!(
    bupdate,
    update2_depth2,
    update4_depth4,
    update16_depth8,
    update32_depth12,
    update128_depth16
);
criterion_main!(bkeygen, bsign, bverify, bupdate);
