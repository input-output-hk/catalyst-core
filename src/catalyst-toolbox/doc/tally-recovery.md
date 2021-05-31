# Recovery of the corrected vote tally from fragment logs

In the Fund 4 time frame, we've been unable to completely resolve issues
with the Catalyst app usage of spending counters in witnesses of successive
ballot (aka VoteCast) transactions for the single voting account.
This may lead to ballot transactions legitimately submitted by voting users
to be rejected by the consensus leader nodes at block creation.

To allow computation of the tally across all ballots legitimately submitted
by the users in face of such discrepancies, an alternative
tallying method was devised and implemented in the catalyst-toolbox tool.
Afterwards, this method can be used to confirm the blockchain tally did not
miss any received legitimate ballots that were recorded into persistent
fragment logs.

## Input: persistent fragment logs

The node or nodes serving REST requests from the clients submitting ballots
has collection of persistent fragment logs enabled. These log files record all
blockchain fragments received by the node and admitted to its mempool,
meaning that the fragments are deduplicated, correctly formatted, and pass some
minimal sanity checks, but otherwise their validity for inclusion into a block
is not verified. The timestamp of fragment arrival is stored along with the
fragment; in the alternative tallying process described here, the chronological
order of fragments is used to decide which choice applies when multiple ballots
for the same proposal have been received from a single account. The timestamp
is also used instead of the block date to check if the ballot is accounted
within the declared voting period.

The format of the log entries is bincode serialization of a Rust structure
containing the unsigned integer timestamp in seconds since the Unix epoch,
and the fragment serialized in the binary blockchain format: 

```rust
pub struct FragmentLogEntry {
    /// The time this fragment was registered and accepted by the pool
    pub time: SecondsSinceUnixEpoch,
    /// Fragment body serialized as byte array
    pub fragment: RawFragment,
}
```

## Initial state in block0

To set up the initial state for tallying, the tool parses the
genesis block of the Catalyst blockchain. It is assumed that the account
balances of registered voters are created in the initial fund distribution
in the genesis block. Likewise, it is assumed that all vote plans on the
blockchain are already present in the genesis block. The initial ledger state
is recreated from the genesis block just like in the blockchain node
(with some technical twists: a mirror set of stake owners' accounts is created
because we need some private keys to supply to the library code performing
ledger state transitions).

## Replay of transactions

The tally recovery command parses the persistent fragment logs and applies
the fragments in order they were received. The processing aims to replay
changes in the ledger's account and vote plan states, as they would be
applied by the blockchain consensus if the spending counters used
to sign consecutive transactions spending from one account were incremented
in order of submission of the fragments by the client.

Transaction validity checks are performed just like in a blockchain node
validating transactions for a block, with the following exceptions:

* The witness signature check for account inputs is performed repeatedly
with multiple candidate values of the spending counter, until a matching
signature is found or the search range is exhausted. The search is started
at the next expected counter value for the account as per the normal witness
validation rule, and proceeds with incrementing distance above and below
the starting value. The search limit is taken to be several times the total
number of proposals in all vote plans. This ensures that transactions
submitted by a participant in control of the account's private key, but
with out-of-order values of the spending counter, are processed as valid.

* As the recovery tool lacks information about blockchain time, it uses
the timestamp in the fragment log entry to check whether the ballot transaction
falls within the voting time period declared in the vote plan. This creates a
small possibility for ballots to be counted by the recovery tool, while
being rejected by the blockchain consensus because they were processed too
late by the slot leader. Likewise, ballots submitted just before voting starts
may be rejected by the recovery tool, but admitted by the blockchain.
The likelihood of occurrence of such edge cases is considered to be low enough
to ignore this potential discrepancy.

Potential replay attacks are prevented by keeping a record of all fragment
hashes encountered and rejecting any duplicate fragments. It is assumed that
the likelihood of a client submitting two legitimate ballots with the same
proposal and choice and the same (incorrectly used) spending counter value
is low enough to ignore such occurrences.

To simplify the logic, the recovery tool only processes fragments of
these kinds:

- Ballot (VoteCast) transactions with one account input and no outputs.
  Private votes are not yet supported.
- Plain value transfer transactions with one account input and one account
  output. The recovery tool updates the voting power with the results of
  the transfer and adds the output accounts.
- VoteTally transactions, processed as a signal to reveal the tally results.

Other kinds of fragments are not expected to be submitted by voting users
or committee members, so the recovery tool only reports such fragments
in warning messages. The party performing the tally recovery should examine
the atypical fragments to decide if they could affect the recovered tally.

## Output: Tally

After the fragment logs have been replayed, the vote plan status including
the tally results is updated as it would be in the node's ledger state,
and printed on the standard output.
The format is chosen to mimic the output of the REST API response for vote
plan status used to extract the blockchain tally.
