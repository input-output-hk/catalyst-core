# Catalyst Event Database

This crate defines the structure and RUST access methods for the Catalyst Event Database.

## Starting a Local Test DB with Docker

If you are not running postgresql-14 locally.
A test server can be run using docker-compose.

```sh
docker-compose -f ./setup/dev-db.docker-compose.yml up --remove-orphans -d
```

This will run postgres on port `5432`, and an `adminer` UI on `localhost:8080`.

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

This can be done simply with:

```sh
psql -e -U postgres -f setup/dev-db.sql
```

Execute Migrations:

```sh
refinery migrate -c refinery.toml -p ./migrations
```

or

```sh
cargo make run-event-db-migration
```

### Setup a clean new dev DB with a single command

```sh
cargo make local-event-db-setup
```

## GraphQL

GraphQL is ONLY used for the admin interface.
It is configured with the `setup/graphql-setup.sql` and the server can be run for local testing with:

```sh
setup/start-graphql.sh
```

See <https://www.graphile.org/postgraphile/> for documentation on the GraphQL server.

### GraphQL Users

There are two GraphQL Users:

* `cat_admin`: Full admin access to the database.
* `cat_anon`: Unauthenticated read-only access to the database.

To authenticate, as the `cat_admin` user, execute the `authenticate` mutation.
This will return a Signed JWT Token for the user.
The JWT Token needs to be included as a `bearer` token `Authorization` header in subsequent requests.
Authentication only last 1 Hour, after which a new token must be requested.

When authenticated as the `cat_admin` user, new `cat_admin` users can be registered with the `registerAdmin` mutation.

Further Security Roles or Admin management functions can be added as required.
