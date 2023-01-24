# Transaction State Database

This is NOT a time-series database, it tracks:

* the current state of the sync from the block chain
* all current UTXO's
* any other data it needs to calculate staked-ADA changes as blocks arrive.

It is updated to track current state and is not historical.

This state is updated atomically, along with:

* The staked ADA database
* The registration database

This ensures that the DB is always in-sync with discrete minted blocks on the block-chain.
The DB NEVER store a partial block update.

* Data

There is no firm specification of the data that needs to be stored.
It should be adapted to efficiently and quickly allow for the functions of the process to execute.

The ONLY critical information that it contains is the current block last synced.

All other information and the structure of it will need to be decided during implementation.
