# Event Definition Table

This table defines the root data and schedules for all Catalyst events.

## Event Table Diagram

![Event DB Event Table](kroki-graphviz:./db-diagrams/event-db-event.dot)

## Schema

```sql
{{#template ../../../src/event-db/migrations/V2__event_tables.sql}}
```
