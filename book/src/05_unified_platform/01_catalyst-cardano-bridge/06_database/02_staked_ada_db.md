# Staked ADA Database

There will be a Staked ADA Database.
This is a time-series database, which means that updates do not replace old records, they are time-stamped instead.
This allows for the state "at a particular time" to be recovered without recreating it.

## Data

The data needed to be stored in each staked ADA record is:

* The Stake address.
* The time and date the ADA staked/rewarded changed.
  * Derived from the block date/time on Cardano, NOT the time it was detected.
  * IF the staked ADA changed MULTIPLE times in the same block:
    * this record contains the total at the END of all updates.
* The block on the blockchain when this stake address total changed.
  * Allows the transaction/s to be verified against the blockchain.
* The total staked ADA as at this Block.
* The total unpaid-rewards ADA as at this Block.
  * Rewards are earned for stake addresses at the end of epochs.
  * They are specially accounted for and need to be `withdrawn`.
  * This total is the total of all ADA which has been awarded to the stake address but NOT yet withdrawn.

Note: ONLY stake addresses which CHANGE are recorded.

It's possible (probable?) that ONLY one of the total staked ADA or total unpaid rewards ADA will update at a time in a single block.
However, regardless of which total updates, the record must faithfully record the total current at that time for both.

For example:

* Staked ADA starts at `1234` and Unpaid Rewards ADA starts at `95`.
* At 12:43 Staked ADA changes to 1200
  * A record is emitted for `12:43` where `staked-ada` = `1200` and `unpaid-rewards` = `95`.
* At 14:55 Unpaid Rewards changes to 143
  * A record is emitted for `12:43` where `staked-ada` = `1200` and `unpaid-rewards` = `143`.

## Queries

Examples of Common Queries:

### Current Staked ADA

Given:

* `AsAt` - The time the registration must be valid by.
* `Window` - Optional, the maximum age of the registration before `AsAt` (Defaults to forever).
* `Stake Address` - the Stake Address.

Return the MOST current Total ADA and Unpaid Rewards ADA for that address.

### All Current Staked ADA

Given:

* `AsAt` - The time the registration must be valid by.
* `Window` - Optional, the maximum age of the registration before `AsAt` (Defaults to forever).

For each unique stake address:
    Return the MOST current Total ADA and Unpaid Rewards ADA for that address.

### Staked Balances for a period

* `AsAt` - The time the registration must be valid by.
* `Age` - The Oldest record to return.

For the period requested return a list of all staked balances where each record in the list is:

* `date-time` - The time this balance applies to.
* `slot` - The slot on the Cardano blockchain the balance changed in.
* `staked` - The total Staked at this time.
* `rewarded` - The total Unclaimed rewards at this time.
