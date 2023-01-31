# Phase 2

Block storage and restart recovery.

1. Phase 1 +
2. Storing the raw blocks in a Surreal DB.
3. Recover the latest Block tip on start from the blocks stored in the DB.
4. Properly handle rollback events from the client by deleting data in the DB and restarting sync.
5. Properly handling a de-sync where the next received block is not the expected block.

## Deliverables

A service that does the above

## Results

Allows us to test recovery and re-sync capabilities on rollback.
