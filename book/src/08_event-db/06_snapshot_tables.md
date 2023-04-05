# Voter Voting Power Snapshot and Vote Storage Table

This table stores:

* The details of each registration and voting power of each voter.
* The results of the latest snapshots for each event.
* The record of all votes cast by voters.

## Snapshot & Vote Table Diagram

![Event DB Snapshot Table](kroki-graphviz:./db-diagrams/event-db-snapshot-vote.dot)

## Snapshot Schema

```sql
{{#template ../../../src/event-db/migrations/V6__snapshot_tables.sql}}
```

## Vote Schema

```sql
{{#template ../../../src/event-db/migrations/V7__vote_tables.sql}}
```
