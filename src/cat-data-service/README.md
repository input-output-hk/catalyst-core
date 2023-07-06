# Catalyst Event Data service

Catalyst event data service

## Build
```
cargo build -p cat-data-service
```
To enable jormungandr mocked `/api/v1/fragments`, `/api/v1/votes/plan/account-votes/{account_id}` endpoints build with the `jorm-mock` feature flag
```
cargo build -p cat-data-service --features jorm-mock
```

## Run
Before running `cat-data-service` you will need to spin up event-db. 
How to do it, you can read this [README.md](https://github.com/input-output-hk/catalyst-core/blob/main/src/event-db/Readme.md#starting-a-local-test-db-with-docker).

Run
```
cat-data-service run --address "127.0.0.1:3030" --database-url=postgres://catalyst-event-dev:CHANGE_ME@localhost/CatalystEventDev --log-level=debug --metrics-address "127.0.0.1:3031"
```
