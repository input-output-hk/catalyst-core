
# Mock Farm

Mock farm is a simple extension for mock service.
It allows to run more that one mock at once and give more control to user in term of starting and stopping particular mock instance.

## Configuration

This section describe configuration file which can be passed as argument for snapshot service:

- `port`: port on which registration-service will be exposed,
- `working_directory`: path to folder which config files will be dumped,
- `mocks-port-range`: range of ports assigned for usage,
- `protocol`: decide whether mock farm should be exposed as http or https,
- `local`: should service be exposed on all network interfaces or only 127.0.0.1,
- `token`: token limiting access to environment. Must be provided in header `API-Token` for each request

Note: it is recommended to run command from `vit-testing/vitup` folder (then no explicit paths are required to be provided).
Configuration file example is available under `vit-testing/vitup/example/mock-farm/config.yaml`

## Start

`vitup start mock-farm --config example\mock\mock-farm\config.yaml`

## Documentation

- [OpenApi](../api/vitup/mock-farm/v0.yaml)
- [Requests collection](../api/vitup/mock-farm/postman_collection.json)
