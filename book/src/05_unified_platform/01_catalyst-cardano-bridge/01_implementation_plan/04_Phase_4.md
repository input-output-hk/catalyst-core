# Phase 4

1. When storing blocks in the DB, check if the block is already stored.
2. If the block is stored and matches the current block, do nothing.
3. If the block is stored and DOES NOT match.
   1. Trigger a re-sync.
   2. Delete the mismatched current block in the DB and all future blocks.
   3. Store the current block.
4. Ensure that 2 services running on the same database can record data securely.

## Deliverables

The service works this way

## Results

The baseline for fully secure sync, rollback and multiple redundant services running in parallel.
