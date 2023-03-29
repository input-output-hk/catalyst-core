# VitSS Compatibility

Compatibility with the legacy VitSS service is being provided via views on the new Catalyst Event Database.
Eventually, these views will be eliminated when VitSS is finally sunset.

## VitSS Compatibility Diagram

![Event DB VitSS Compatibility](kroki-graphviz:./db-diagrams/event-db-vitss-views.dot)

## Schema

```sql
{{#template ../../../src/event-db/migrations/V9__vitss_compatibility.sql}}
```
