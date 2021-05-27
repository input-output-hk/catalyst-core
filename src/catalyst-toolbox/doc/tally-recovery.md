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
for the same proposal have been received from a single account.

## Initial state: voter accounts in block0

To determine which accounts can legitimately vote, the tool parses the
genesis block of the Catalyst blockchain. It is assumed that the registered
voters' account balances are created in the initial fund distribution in the
genesis block. The initial ledger state is recreated from the genesis block
just like in the blockchain node (with some technical twists: a
mirror set of accounts is created because we need some private keys to
supply to the library code performing ledger state transitions).

## Replay of transactions

The tally recovery command parses the persistent fragment logs and applies
the fragments in order they were received, to replay changes in the ledger state
of a blockchain node that would process transactions to the same effect, if
the account spending counters used to sign transactions spending form the same
account incremented in order of submission by the client.

Transaction validity checks are performed just like in the blockchain node
validating transactions for a block, with the exception that the witness
signature check for account inputs is performed repeatedly with multiple
candidate values of the spending counter until a matching signature is found
or the search range is exhausted, searching with increasing difference
above and below the next expected counter value for the account
as per the normal transaction validation rule. This ensures that transactions
submitted by a participant in control of the account private key, but
with out-of-order values of the spending counter, are processed as valid.

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

Other kinds of fragments are not expected to be submitted by voting users,
so the recovery tool only reports such fragments in warning messages.
The party performing the tally recovery should to examine the atypical
fragments to decide if they could affect the recovered tally.

## Output: Tally

After the fragment logs have been replayed, the vote plan status including
the tally results is updated as it would be in the node's ledger state,
and printed on the standard output.
The format is chosen to mimic the output of the REST API response for vote
plan status used to extract the blockchain tally.
