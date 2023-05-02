# Catalyst Event Database

This crate defines the structure and RUST access methods for the Catalyst Event Database.

- [Catalyst Event Database](#catalyst-event-database)
  - [Starting a Local Test DB with Docker](#starting-a-local-test-db-with-docker)
  - [Creating A Local Test Database](#creating-a-local-test-database)
    - [Setup a clean new dev DB with a single command](#setup-a-clean-new-dev-db-with-a-single-command)
  - [GraphQL](#graphql)
    - [GraphQL Users](#graphql-users)
      - [Authentication API](#authentication-api)

## Starting a Local Test DB with Docker

If you are not running postgresql-14 locally.
A test server can be run using docker-compose.

```sh
docker-compose -f ./setup/dev-db.docker-compose.yml up --remove-orphans -d
```

This will run postgres on port `5432`, and an `adminer` UI on `localhost:8080`.

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

Or (you need to run these scripts from the root folder)

```sh
cargo make local-event-db-init
```

Execute Migrations:

```sh
cargo make run-event-db-migration
```

### Setup a clean new dev DB with a single command

```sh
cargo make local-event-db-setup
```

## GraphQL

GraphQL is ONLY used for the admin interface.
It is configured with the `setup/graphql-setup.sql`.

For local testing, make sure the local `event-db` is setup and running, as described above.
Then:

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

#### Authentication API

The GraphQL exposes 2 **Mutations** which are used for security.

If the user is NOT authenticated, they can not update any data in the database, only read data.

To authenticate run the `authenticate` mutation with the email address and password of the user to be authenticated.
If successful, this mutation will return a JWT which will have 1 hr of Life.
Place the returned JSW in a `Authorization: Bearer <JWT>` header for all subsequent calls.

`currentAcct` query will return the authenticated users current account details and role.

To register a new user, the `registerAdmin` mutation can be used.
It will only work if the user is properly authenticated with a `cat_admin` role.
