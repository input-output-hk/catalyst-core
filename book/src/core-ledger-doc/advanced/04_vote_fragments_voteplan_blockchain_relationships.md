# How Vote plans, Vote Fragments and the block chain transaction work and inter-relate

Please just brain dump everything you know about the above topics, or anything
related to them, either individually or interrelated.  This process is not
intended to consume an excessive amount of your time, so focus more on getting
the information you have to contribute down in the quickest way possible.

Don't be overly concerned with format or correctness,  its not a test.  If you
think things work in a particular way, describe it.  Obviously, different people
will know different things, don't second guess info and not include it because
you think someone else might say it.

If you have technical details, like the format of a data entity that can be
explained, please include it.  This is intended to become a deep dive, to the
byte level.  If you want to,  feel free to x-ref the code as well.

Add what you know (if anything) in the section below your name and submit a PR
to the DOCS branch (not main) with Steven Johnson for review.  I will both
review and merge these.  I will also start collating the data once this process
is complete, and we can then iterate until the picture is fully formed and
accurate. Feel free to include other .md files if there is a big piece of
information, such as the format of a vote transaction, or the vote plan section
of block 0, etc.  Or refer to other documentation we may already have (in any
form, eg confluence, jira issue or Miro, or the old repos or Anywhere else is
ok.).

We are particularly interested in, with respect to Jormungandr:

1. How the voteplan is set up, and what the various fields of the vote plan are
   and how they are specified.
2. How individual votes relate to vote plans.
3. How votes are prevented from being cast twice by the same voter.
4. The format of the entire vote transaction, both public and private.
5. How is the tally conducted. (is it done in jormungandr, or with the jcli
   tool for example)?
6. Anything else which is not listed but is necessary to fully understand the
   votes cast in jormungandr.

Don't feel limited by this list,  if there is anything else the list doesn't
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

## Cameron Mcloughlin

## Dariusz Kijania

## Ognjen Dokmanovic

## Stefan Rasevic
