# Catalyst Event Database

This crate defines the structure and RUST access methods for the Catalyst Event Database.

## Creating A Local Test Database

Run the following SQL on your local test PostgreSQL server:

```sql
-- Cleanup if we already ran this before.
drop database if exists "CatalystEventDev";
drop user if exists "catalyst-event-dev";

-- Create the test user we will use with the local Catalyst-Event dev database.
create user "catalyst-event-dev" with password 'CHANGE_ME';

-- Create the database.
create database "CatalystEventDev"
    with owner "catalyst-event-dev";

comment on database "CatalystEventDev" is 'Local Test Catalyst Event DB';
```

Execute Migrations:

```sh
refinery migrate -c refinery.toml -p ./migrations
```

or

```sh
cargo make run-event-db-migration
```
