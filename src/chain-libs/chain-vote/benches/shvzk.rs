use chain_vote::*;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::Rng;
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;

fn common(rng: &mut ChaCha20Rng) -> ElectionPublicKey {
    let h = Crs::from_hash(&[0u8; 32]);

    let mc1 = MemberCommunicationKey::new(rng);
    let mc = [mc1.to_public()];

    let threshold = 1;

    let m1 = MemberState::new(rng, threshold, &h, &mc, 0);

    let participants = vec![m1.public_key()];
    ElectionPublicKey::from_participants(&participants)
}

fn encrypt_and_prove(c: &mut Criterion) {
    let mut rng = ChaCha20Rng::from_seed([0u8; 32]);
    let mut group = c.benchmark_group("Encrypt and prove");
    let crs = Crs::from_hash(&[0u8; 32]);
    let ek = common(&mut rng);

    for &number_candidates in [2usize, 4, 8, 16, 32, 64, 128, 256, 512].iter() {
        let parameter_string = format!("{} candidates", number_candidates);
        group.bench_with_input(
            BenchmarkId::new("Encrypt and Prove", parameter_string),
            &number_candidates,
            |b, &nr| {
                b.iter(|| ek.encrypt_and_prove_vote(&mut rng, &crs, Vote::new(nr, 0).unwrap()))
            },
        );
    }

    group.finish();
}

fn prove(c: &mut Criterion) {
    let mut rng = ChaCha20Rng::from_seed([0u8; 32]);
    let mut group = c.benchmark_group("Prove encrypted vote");
    let crs = Crs::from_hash(&[0u8; 32]);
    let ek = common(&mut rng);

    for &number_candidates in [2usize, 4, 8, 16, 32, 64, 128, 256, 512].iter() {
        group.bench_with_input(
            BenchmarkId::new("Prove with", format!("{} candidates", number_candidates)),
            &{
                let vote =
                    Vote::new(number_candidates, rng.gen_range(0..number_candidates)).unwrap();
                (vote, ek.encrypt_vote(&mut rng, vote))
            },
            |b, (vote, (vote_enc, randomness))| {
                b.iter(|| ek.prove_encrypted_vote(&mut rng, &crs, *vote, vote_enc, randomness))
            },
        );
    }

    group.finish();
}

fn verify(c: &mut Criterion) {
    let mut rng = ChaCha20Rng::from_seed([0u8; 32]);
    let mut group = c.benchmark_group("Verify vote proof");
    let crs = Crs::from_hash(&[0u8; 32]);
    let ek = common(&mut rng);

    for &number_candidates in [2usize, 4, 8, 16, 32, 64, 128, 256, 512].iter() {
        let (vote, proof) =
            ek.encrypt_and_prove_vote(&mut rng, &crs, Vote::new(number_candidates, 0).unwrap());
        let parameter_string = format!("{} candidates", number_candidates);
        group.bench_with_input(
            BenchmarkId::new("Verify with", parameter_string),
            &number_candidates,
            |b, _| b.iter(|| proof.verify(&crs, ek.as_raw(), &vote)),
        );
    }

    group.finish();
}

criterion_group!(
    name = shvzk;
    config = Criterion::default().sample_size(500);
    targets =
    encrypt_and_prove,
    prove,
    verify,
);

criterion_main!(shvzk);
