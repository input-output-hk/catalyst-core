# Catalyst Event Data service

Catalyst event data service

## Build
```
cargo build -p cat-data-service
```
To enable jormungandr mocked `/api/v1/fragments`, `/api/v1/votes/plan/account-votes/{account_id}` endpoints build with the `jorm-mock` feature flag. These endpoints have a time limit for internal state, after fragment has been submitted it will be cleanup from the internal after 10 minutes timeout by default
```
cargo build -p cat-data-service --features jorm-mock
```

## Run
Before running `cat-data-service` you will need to spin up event-db. 
How to do it, you can read this [README.md](https://github.com/input-output-hk/catalyst-core/blob/main/src/event-db/Readme.md#starting-a-local-test-db-with-docker).
To run with the specific jorm mock state cleanup timeout you can specify `JORM_CLEANUP_TIMEOUT` env var.

Run
```
cat-data-service run --address "127.0.0.1:3030" --database-url=postgres://catalyst-event-dev:CHANGE_ME@localhost/CatalystEventDev --log-level=debug --log-format=compact --metrics-address "127.0.0.1:3031"
```

