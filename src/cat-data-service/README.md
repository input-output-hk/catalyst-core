# Catalyst Event Data service

Catalyst event data service

## Run
Before running `cat-data-service` you will need to spin up event-db. 
How to do it, you can read this [README.md](https://github.com/input-output-hk/catalyst-core/blob/main/src/event-db/Readme.md#starting-a-local-test-db-with-docker).

Run
```
cat-data-service run --address "127.0.0.1:3031" --database-url=postgres://catalyst-event-dev@localhost/CatalystEventDev --log-level=debug
```
