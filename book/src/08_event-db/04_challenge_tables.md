# Challenge and Proposal  Tables

These tables define the data known about Challenges and Proposals.

## Challenge and Proposal Table Diagram

![Event DB Event Table](kroki-graphviz:./db-diagrams/event-db-challenge-proposal.dot)

## Challenge Schema

```sql
{{#template ../../../src/event-db/migrations/V3__challenge_tables.sql}}
```

## Proposal Schema

```sql
{{#template ../../../src/event-db/migrations/V4__proposal_tables.sql}}
```
