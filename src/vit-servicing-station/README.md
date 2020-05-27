# VIT Servicing Station

--------------

VIT as a service (VaaS)

--------------


### Building tips and tricks

We use [`diesel`](http://diesel.rs/) for database (`sqlite3`) integration. Please refer to the [`diesel_cli` documentation](https://docs.rs/crate/diesel_cli/) to understand how to operate with migrations and setup.

Diesel generates rust code based on a *SQL* migration script (`/migrations/*/up.sql`) when running the migration with `diesel_cli`.
Diesel code generation is configured in the `diesel.toml` file. Right now it just contains the path on where the rust code should be generated.

Another file to look at is the `.env` file. This file holds the environment variables used by this project.
For example, `diesel` uses a `DATABASE_URL` variable to know where should he generate the database file. 
It also may be used by the binary itself to load the same information and have both generated items and configuration synced.