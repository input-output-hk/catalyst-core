# Voting Tools (Rust)

This tool generates voting power info from a db-sync instance.

Example usage:
```
snapshot-tool --db postgres --db-user postgres --db-host localhost --out-file output.json
```

## Building

Building with nix should be straightforward, simply enter a dev environment with `nix develop`, then run `cargo build` to build.

## Testing

Most tests run via `cargo test`. To run all tests, pass `--features all_tests`. At present, this enables the `db_tests` feature, which is a bit tricky to run.

### Database tests

Database tests perform predefined queries against a test database. If the results don't match the snapshots, the test fails. This requires having the correct data in your database. The current test data can be found [here](https://updates-cardano-testnet.s3.amazonaws.com/cardano-db-sync/index.html#13/)

Once you have this database set up, create a file at `<project_root>/test_db.json`, which contains credentials to connect to this database, for example:
```json
{
  "host": "localhost",
  "name": "database_name",
  "user": "username",
  "password": "password"
}
```
(Note, password is optional).

From there, running `cargo test --features all_tests` will run database tests as well as regular tests. If tests pass, great!

If not, you need to review the changes. It's possible that you intended to change the result of a query. Use `cargo insta review` to go through all failed tests and mark them as "intended" or not.
