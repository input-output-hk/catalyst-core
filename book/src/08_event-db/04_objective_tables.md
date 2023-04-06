# Objective and Proposal  Tables

These tables define the data known about Challenges and Proposals.

## Objective and Proposal Table Diagram

![Event DB Event Table](kroki-graphviz:./db-diagrams/event-db-objective-proposal.dot)

## Objective Schema

```sql
{{#template ../../../src/event-db/migrations/V3__objective_tables.sql}}
```

## Proposal Schema

```sql
{{#template ../../../src/event-db/migrations/V4__proposal_tables.sql}}
```
