# How Vote plans, Vote Fragments and the blockchain transaction work and inter-relate

Please just brain dump everything you know about the above topics, or anything
related to them, either individually or interrelated. This process is not
intended to consume an excessive amount of your time, so focus more on getting
the information you have to contribute down in the quickest way possible.

Don't be overly concerned with format or correctness, its not a test. If you
think things work in a particular way, describe it. Obviously, different people
will know different things, don't second guess info and not include it because
you think someone else might say it.

If you have technical details, like the format of a data entity that can be
explained, please include it. This is intended to become a deep dive, to the
byte level. If you want to, feel free to x-ref the code as well.

Add what you know (if anything) in the section below your name and submit a PR
to the DOCS branch (not main) with Steven Johnson for review. I will both
review and merge these. I will also start collating the data once this process
is complete, and we can then iterate until the picture is fully formed and
accurate. Feel free to include other .md files if there is a big piece of
information, such as the format of a vote transaction, or the vote plan section
of block 0, etc. Or refer to other documentation we may already have (in any
form, eg confluence, jira issue or Miro, or the old repos or Anywhere else is
ok.).

For Jormungandr, we are particularly interested in:

How the vote plan is set up, what the various fields of the vote plan are
   and how they are specified.
2. How individual votes relate to vote-plans.
3. How votes are prevented from being cast twice by the same voter.
4. The format of the entire vote transaction, both public and private.
5. How is the tally conducted? (is it done in Jormungandr, or with the jcli
   tool for example)?
6. Anything else which is not listed but is necessary to fully understand the
   votes cast in Jormungandr.

Don't feel limited by this list, if there is anything else the list doesn't
cover but you want to describe it, please do.

## Sasha Prokhorenko

## Nicolo Padovani

## Felipe Rosa

## Joaquin Rosales

### `Proposal.chain_proposal_id`

This field is not very well documented, except for a line in `book/src/core-vitss-doc/api/v0.yaml` that describes it as:

    > Identifier of the proposal on the blockchain.

Internally, the identifier is of type `ExternalProposalId` (`src/chain-libs/chain-impl-mockchain/src/certificate/vote_plan.rs`) which is an alias type for `DigestOf<Blake2b256, _>`, from the `chain_crypto` crate. This is undocumented.

The `ExternalProposalId` is sent through the wire and csv files as a 64-character hex-encoded string.

The `catalyst-toolbox` binary decodes this hex string, and converts it into a valid `ExternalProposalId` so that the underlying `[u8; 32]` can be extracted, hashed and used in logic related to rewards thresholds, votes, and dreps.

There is an arbitrary snapshot generator used in `vit-servicing-station-tests` that creates valid `ExternalProposalId` from a randomized `[u8; 32]` array that is used in integration tests found in `vit-testing`.

## Stefano Cunego

## Conor Gannon

## Alex Pozhylenkov

### `Spending Counters`

Spending counter associated to an account. Every time the owner is spending from an account, the counter is incremented. This features is similar to the Ethereum `nonce` field in the block and prevents from the replay attack.

```
pub struct SpendingCounter(pub(crate) u32);
```

As it was said before every account associated with the a current state of the Spending Counter, or to be more precised to an array of 8 Spending counters.

```
pub struct SpendingCounterIncreasing {
    nexts: Vec<SpendingCounter>,
}
```

Each spending counter differes with each other with the specified `lane` bits which are a first 3 bits of the original Spending counter value. Spending counter structure:

```
(001)[lane] (00000 00000000 00000000 00000001){counter}
(00100000 00000000 00000000 00000001){whole Spending Counter}
```

With such approach user can generate up to 8 transactions with the specified different lanes and correspoding counters and submit it into the blockchain with no mater on the transaction processing order. So incrementing of the counter will be done in "parallel" for each lane. That is the only difference with the original Ethereum approach with `nonce` (counter in our implementation), where for each transaction you should specify an exact value and submits transaction in the exact order.

## Cameron Mcloughlin

## Dariusz Kijania

## Ognjen Dokmanovic

## Stefan Rasevic
