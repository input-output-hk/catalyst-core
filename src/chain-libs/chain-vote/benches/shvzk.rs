use chain_vote::debug::{gang, shvzk};
use chain_vote::*;
use criterion::{criterion_group, criterion_main, Criterion};
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;

fn common(rng: &mut ChaCha20Rng) -> (EncryptingVoteKey, EncryptingVote) {
    let h = gang::GroupElement::random(rng);

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

fn generate(c: &mut Criterion) {
    let mut rng = ChaCha20Rng::from_seed([0u8; 32]);
    let (ek, ev) = common(&mut rng);
    c.bench_function("generate", |b| {
        b.iter(|| shvzk::prove(&mut rng, ek.as_raw(), ev.clone()))
    });
}

fn verify(c: &mut Criterion) {
    let mut rng = ChaCha20Rng::from_seed([0u8; 32]);
    let (ek, ev) = common(&mut rng);
    let proof = shvzk::prove(&mut rng, ek.as_raw(), ev.clone());
    c.bench_function("verify", |b| {
        b.iter(|| shvzk::verify(&ek.as_raw(), &ev.ciphertexts, &proof))
    });
}

criterion_group!(shvzk, generate, verify);
criterion_main!(shvzk);
