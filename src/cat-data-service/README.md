# Catalyst Event Data service

Catalyst event data service

## Run
Run with the default address argument
```
cat-data-service run
```
Run specifying address argument
```
cat-data-service run --address "127.0.0.1:3031
```

## Endpoints
`/api/v0/snapshot/voter/`
```
curl --request GET 127.0.0.1:3030/api/v0/snapshot/voter/latest/voter1
```
`/api/v0/snapshot/delegator/`
```
curl --request GET 127.0.0.1:3030/api/v0/snapshot/delegator/latest/delegator1```

