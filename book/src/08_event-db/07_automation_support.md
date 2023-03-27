# Catalyst Automation Support Tables

This table defines the data necessary to support continuous automation of Catalyst 1.0 Backend.

## Event Table Diagram

![Event DB Event Table](kroki-graphviz:./db-diagrams/event-db-automation.dot)

## Schema

```sql
{{#template ../../../src/event-db/migrations/V8__catalyst_automation.sql}}
```
