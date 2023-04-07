# Catalyst Event Database

This crate defines the structure and RUST access methods for the Catalyst Event Database.

## Creating A Local Test Database

### Dependencies
 - `cargo-make`, install `cargo install cargo-make`
 - `refinery`, install `cargo install refinery_cli`

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
Or
```sh
cargo make local-event-db-init
```

Execute Migrations:
```sh
cargo make run-event-db-migration
```


