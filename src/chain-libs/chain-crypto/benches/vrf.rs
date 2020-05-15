use chain_crypto::algorithms::vrf::vrf::{PublicKey, SecretKey};
use criterion::{criterion_group, criterion_main, Criterion};
use rand_core::{OsRng, RngCore};

fn common() -> (OsRng, SecretKey, PublicKey, [u8; 10], [u8; 10]) {
    let mut csprng: OsRng = OsRng;
    let sk = SecretKey::random(&mut csprng);
    let pk = sk.public();

    let sk_other = SecretKey::random(&mut csprng);
    let _pk_other = sk_other.public();

    let mut b1 = [0u8; 10];
    for i in b1.iter_mut() {
        *i = csprng.next_u32() as u8;
    }
    let mut b2 = [0u8; 10];
    for i in b2.iter_mut() {
        *i = csprng.next_u32() as u8;
    }

    (csprng, sk, pk, b1, b2)
}

fn generate(c: &mut Criterion) {
    let (mut csprng, sk, _pk, b1, _) = common();

    c.bench_function("generate", |b| {
        b.iter(|| {
            let _ = sk.evaluate_simple(&mut csprng, &b1[..]);
        })
    });
}

fn verify_success(c: &mut Criterion) {
    let (mut csprng, sk, pk, b1, _) = common();
    let po = sk.evaluate_simple(&mut csprng, &b1[..]);

    c.bench_function("verify_success", |b| {
        b.iter(|| {
            let _ = po.verify(&pk, &b1[..]);
        })
    });
}

fn verify_fail(c: &mut Criterion) {
    let (mut csprng, sk, _pk, b1, _b2) = common();
    let (_, _, pk2, _, _) = common();
    let po = sk.evaluate_simple(&mut csprng, &b1[..]);

    c.bench_function("verify_fail", |b| {
        b.iter(|| {
            let _ = po.verify(&pk2, &b1[..]);
        })
    });
}

criterion_group!(vrf, generate, verify_success, verify_fail);
criterion_main!(vrf);
