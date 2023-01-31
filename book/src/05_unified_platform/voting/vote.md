# Catalyst voting crate

## Actors

The main actors of the voting protocol are the following:
- Voter: an actor who makes a decision about a particular proposal submitted for ratification
- Expert - can have multiple roles
    - dRep: A person who you delegate your voting power to.
    - Proposal Assessor/Veteran Proposal Assessor : Someone who rates proposals for merit based on their expertise.
- Committee members: special actors who maintain the voting procedure (generate distributed encryption key, do joint decryption of the tally, etc.)

## Ingredients

The voting protocol has the following ingredients:

- Basic cryptographic protocols (ElGamal encryption, hybrid encryption, wrappers for crypto primitives)
- A set of non-interactive zero-knowledge proofs
- Distributed Key Generation Protocol
- Ballots encryption
- Joint decryption
- Randomness generation
- Several types of voting systems (approval voting, preferential voting)

## Context

The philosophy of the voting protocol is implementation-agnostic. 

> The implemented voting protocol can be used not only for blockchain systems. It can be successfully reused, for instance, to deploy a secure decentralized fault tolerant voting scheme in some private network where participating entities communicate directly with each other instead of using a blockchain as a channel.

## Goal

JOR implementation contains all of the ingredients but is tightly coupled to the underlying blockchain implementation.

Dissect, decouple and organize voting protocol implementation that currently resides in JOR and transplant to new voting crate in neo.

- Extract and isolate Cryptography primitives
- Compose primitives into voting protocol

## Vote fundamentals API

### Create Ballot

- After the preparation stage there are a set of proposals P := {P1, . . . , Pk}
- During the voting stage, voters and experts issue voting ballots where they put their choices regarding proposals.
- Voters are defined as a set of stake holders that deposited a certain amount of stake to
participate in voting; the voting power is proportional to the amount of deposited stake.
- During the voting stage, voters and experts issue voting ballots where they put their choices regarding
proposals. For each proposal, a voter may chose among three options: Yes, No, Abstain,
- a vote is an ordered list of proposal ids depending on their priorities


```rust

/// E.g.: given a 3 possible votes in the 0-indexed set {option 0, option 1, option 2}, then
/// the vote "001" represents a vote for "option 2"
pub type Vote = UnitVector;

/// Encrypted vote is a unit vector where each element is an ElGamal Ciphertext, encrypted with
/// the Election Public Key.
pub type EncryptedVote = Vec<Ciphertext>;

/// A proof of correct vote encryption consists of a unit vector zkp, where the voter proves that
/// the `EncryptedVote` is indeed a unit vector, and contains a vote for a single candidate.
pub type ProofOfCorrectVote = UnitVectorZkp;

fn create_ballot(vote: Vote, pub_key: ElectionPublicKey, stake)-> (EncryptedVote, ProofOfCorrectVote) {

}

is JOR implementation using stake in crypto? Seems not.

```

### Distributed Key Generation
During the DKG phase, the elected voting committee jointly generates a shared public voting key which will be used by voters and experts to encrypt their ballots.

```rust

/// The overall committee public key used for everyone to encrypt their vote to.
pub struct ElectionPublicKey(pub(crate) PublicKey);

fn generate_election_pub_key(pks: &[MemberPublicKey])-> ElectionPublicKey{

}
```

### Joint Decryption
TODO!


