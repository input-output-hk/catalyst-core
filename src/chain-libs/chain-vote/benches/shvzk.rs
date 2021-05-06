use chain_vote::*;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;

fn common(rng: &mut ChaCha20Rng) -> (EncryptingVoteKey, EncryptingVote) {
    let h = Crs::from_hash(&[0u8; 32]);

    let mc1 = MemberCommunicationKey::new(rng);
    let mc = [mc1.to_public()];

    let threshold = 1;

    let m1 = MemberState::new(rng, threshold, &h, &mc, 0);

    let participants = vec![m1.public_key()];
    let ek = EncryptingVoteKey::from_participants(&participants);

    let vote_options = 2;
    let vote = Vote::new(vote_options, 0);

    let ev = EncryptingVote::prepare(rng, ek.as_raw(), &vote);
    (ek, ev)
}

fn encrypt_and_prove(c: &mut Criterion) {
    let mut rng = ChaCha20Rng::from_seed([0u8; 32]);
    let mut group = c.benchmark_group("Encrypt and prove");
    let crs = Crs::from_hash(&[0u8; 32]);
    let (ek, _) = common(&mut rng);

    for &number_candidates in [2usize, 4, 8].iter() {
        let parameter_string = format!("{} candidates", number_candidates);
        group.bench_with_input(
            BenchmarkId::new("Encrypt and Prove", parameter_string),
            &number_candidates,
            |b, &nr| b.iter(|| encrypt_vote(&mut rng, &crs, &ek, Vote::new(nr, 0))),
        );
    }

    group.finish();
}

fn verify(c: &mut Criterion) {
    let mut rng = ChaCha20Rng::from_seed([0u8; 32]);
    let mut group = c.benchmark_group("Verify vote proof");
    let crs = Crs::from_hash(&[0u8; 32]);
    let (ek, _) = common(&mut rng);

    for &number_candidates in [2usize, 4, 8].iter() {
        let (vote, proof) = encrypt_vote(&mut rng, &crs, &ek, Vote::new(number_candidates, 0));
        let parameter_string = format!("{} candidates", number_candidates);
        group.bench_with_input(
            BenchmarkId::new("Verify with", parameter_string),
            &number_candidates,
            |b, _| b.iter(|| verify_vote(&crs, &ek, &vote, &proof)),
        );
    }

    group.finish();
}

criterion_group!(
    name = shvzk;
    config = Criterion::default();
    targets =
    encrypt_and_prove,
    verify,
);

criterion_main!(shvzk);
