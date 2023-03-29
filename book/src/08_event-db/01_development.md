# Event DB Local Development

The Event DB is targeted to a Postgresql Database, and for local development,
an instance must be running.

## Installing Postgresql

Please see your operating systems' specific guides for installing and configuring
Postgresql.

## Initialize the Database

After Postgres is installed, as the `postgres` user:

1. Init the Database (if it doesn't exist already):
    The recommended database initialization command (on Linux or Mac) is:

    ```sh
    [postgres@host]$ initdb --locale=C.UTF-8 --encoding=UTF8 -D /var/lib/postgres/data --data-checksums
    ```

2. Create a Development user

    ```sh
    [postgres@host]$ createuser -P catalyst-dev
    ```

    when prompted, enter a password, eg "`CHANGE_ME`"

3. Create a Development Database:

    ```sh
    [postgres@host]$ createdb CatalystDev
    ```
