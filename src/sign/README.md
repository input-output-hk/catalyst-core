# **Vote** Fragment generator and signer:
Generates vote fragments and signs them accordingly

## Specifications
 [*see here for format.abnf*](../chain-libs/chain-impl-mockchain/doc/format.abnf)

 [*see here for format.md*](../chain-libs/chain-impl-mockchain/doc/format.md)

## Ingredients for generating a **vote** fragment

- Election public key
- Alice public key
- Alice private key
- proposal to vote on
- vote plan id (hash of voteplan)
- epoch
- slot

*Example usage:*

```
cargo build --release -p sign
```  

*Generate raw vote fragment in byte representation*

```bash

ELECTION_PUB_KEY=ristretto255_votepk1ppxnuxrqa4728evnp2ues000uvwvwtxmtf77ejc29lknjuqqu44s4cfmja
ALICE_SK=56e367979579e2ce27fbd305892b0706b7dede999a534a864a7430a5c6aefd3c
ALICE_PK=ea084d2d80ed0ab681333d934efc56df3868d13d46a2de3b7f27f40b62e5344d
PROPOSAL=5
VOTE_PLAN_ID=36ad42885189a0ac3438cdb57bc8ac7f6542e05a59d1f2e4d1d38194c9d4ac7b
EPOCH=0
SLOT=0
CHOICE=1

./target/release/sign --election-pub-key $ELECTION_PUB_KEY --private-key $ALICE_SK --public-key $ALICE_PK --proposal $PROPOSAL --vote-plan-id $VOTE_PLAN_ID --epoch $EPOCH --slot $SLOT --choice $CHOICE

```