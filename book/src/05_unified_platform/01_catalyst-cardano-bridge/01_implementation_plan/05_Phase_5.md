# Phase 5

While syncing:

1. Scan for CIP-15 Registration transactions in the block.
   1. Validate them.
   2. Record valid transactions.
   3. Record invalid transactions with a validation error for reporting.
   4. Records are kept in a time series database.
2. Scan for CIP-36 Registration transactions
   1. Validate them.
   2. Record valid transactions.
   3. Record invalid transactions with a validation error for reporting.
   4. Records are kept in a time series database.
3. On a rollback, purge registrations as well as blocks.
4. Make sure all updates are atomic per block.
   1. Both the raw block and all registrations in the block are recorded in the DB
   2. Or none are.

## Deliverables

The service running as expected

## Result

A usable service that is properly recording all registrations as they occur in a time series database.
