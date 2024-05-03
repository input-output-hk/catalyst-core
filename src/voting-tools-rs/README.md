# Voting Tools (Rust)

This tool generates voting power info from a `cardano-db-sync` instance.

Example usage:

```sh
snapshot-tool --db postgres --db-user postgres --db-host localhost --out-file output.json
```

To get a full list of available arguments run:

```sh
snapshot-tool --help
```

## Building

Building with nix should be straightforward, simply enter a dev environment with `nix develop`, then run `cargo build -p voting_tools_rs` to build.

## Testing

To run tests, run `cargo test -p voting_tools_rs`.

## Spin up cardano-db-sync

To sucessufully run the `snapshot-tool` it is needed to have a running [`cardano-db-sync`](https://github.com/IntersectMBO/cardano-db-sync) instance.

[Here](https://github.com/IntersectMBO/cardano-db-sync/blob/master/doc/building-running.md) you can found a guide how to build and run `cardano-db-sync`.
