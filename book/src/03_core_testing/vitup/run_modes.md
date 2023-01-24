
# Configuration modes

In order to take out the burden of providing entire configuration vitup has two configuration modes:

- `quick` - which runs on defaults and allow user to override most important parameters using cli arguments:

`vitup start quick`

- `advanced` - which allows to defined full configuration as well as external static files for proposals and challenges

`vitup start advanced`

## Run Modes

There are 4 run modes available in vitup:

- `interactive` - where user can push some fragments or query status of nodes
- `endless` - [Default] just simple run until stopped by user
- `service` - additional manager service will be published at `0.0.0.0:3030`.
  They allow to control (stop/start) and provides resources over http (qr codes or secret keys)
- `mock` - lightweight version of backend with does not spawn any jormungandr or vit-servicing-station services.
  Mock is also capable of controlling more backend aspect than normal deployment (cut off the connections, rejects all fragments.

## Endless mode

There are two ways of starting vitup in endless mode. One with limited configuration and one with giving full control.

`vitup start quick --mode endless ..` or

`vitup start advanced --mode endless ..`

## Service mode

`vitup start quick --mode service ..` or

`vitup start advanced --mode service ..`

Once environment is up one can check status or modify existing environment:

### Admin Operations

- start -  in order to start new voting
- stop -  stops currently running vote backend (usually it takes 1 min to stop it)
- status -  check status of environment:
    1. `Idle` - environment is not started
    2. `Starting` - environment is starting, please wait until its status is `Running`,
    3. `Running` - environment is not running and should be accessible,
    4. `Stopping` - environment is stopping, please wait until its `Idle` to start it with different parameters,

- files:
In order to get qr-codes or secret files from env, two operations are provided:

1. `List Files` - list all files in data directory for current run
2. `Get File` - downloads particular file which is visible in `List Files` operation result

### how to send operations

Voting backend admin console is an REST API accessible over http or https on port 3030.
Using POST/GET http methods admin can send some operations to environment.
There are various apps capable of sending REST commands.
The simplest is to download `Postman` and use UI to fire up commands.

1. Download postman: `https://www.postman.com/downloads/`
2. Review quick guide, how to send dummy request: `https://learning.postman.com/docs/getting-started/sending-the-first-request/`
3. Review guide, how to send different requests with arguments: `https://learning.postman.com/docs/sending-requests/requests/`

Available commands:

### check environment status

- Request Type: GET
- Endpoint : <http://{env_endpoint}:3030/api/control/command/status>
- Response Example:

```text
    Running
```

### start environment

Default parameters:

- Request Type: POST
- Endpoint : <http://{env_endpoint}:3030/api/control/command/start>
- Response Example:

```text
start event received
```

Custom parameters:

- Request Type: POST
- Endpoint : <http://{env_endpoint}:3030/api/control/command/start/default>
- BODY: json with configuration
- Response Example:

```text
    start event received
```

This requests need to pass environment configuration file in Body.

#### stop environment

- Request Type: POST
- Endpoint : `http://{env_endpoint}:3030/api/control/command/stop`
- Response Example:

```text
stop event received
```

### list files

- Request Type: GET
- Endpoint : `http://{env_endpoint}:3030/api/control/files/list`
- Response Example:

```json
{
    "content": {
        "network": [
            "Leader4/node.log",
            "Leader4/node_config.yaml",
            "Leader4/node_secret.yaml",
            "vit_station/vit_config.yaml",
            "initial_setup.dot",
            "Leader1/node.log",
            "Leader1/node_config.yaml",
            "Leader1/node_secret.yaml",
            "Leader3/node.log",
            "Leader3/node_config.yaml",
            "Leader3/node_secret.yaml",
            "Leader2/node.log",
            "Leader2/node_config.yaml",
            "Leader2/node_secret.yaml",
            "Wallet_Node/node.log",
            "Wallet_Node/node_config.yaml",
            "Wallet_Node/node_secret.yaml"
        ],
        "qr-codes": [
            "qr-codes/zero_funds_12_0000.png",
            "qr-codes/wallet_25_above_8000_1234.png",
            "qr-codes/wallet_12_below_8000_9807.png",
            "qr-codes/wallet_30_below_8000_9807.png"
        ],
        "private_keys": [
            "wallet_13_below_8000",
            "wallet_26_below_8000",
            "wallet_23_above_8000",
            "wallet_26_above_8000"
        ],
        "private_data": [
            "fund_3_committees/ed25519_pk192pta739an4q6phkr4v7pxpgna5544mkkfh8ce6p0auxmk5j89xs0706fp/communication_key.sk",
            "fund_3_committees/ed25519_pk192pta739an4q6phkr4v7pxpgna5544mkkfh8ce6p0auxmk5j89xs0706fp/encrypting_vote_key.sk",
            "fund_3_committees/ed25519_pk192pta739an4q6phkr4v7pxpgna5544mkkfh8ce6p0auxmk5j89xs0706fp/member_secret_key.sk"
        ],
        "blockchain": [
            "block0.bin",
            "genesis.yaml"
        ]
    },
    "root": "./vit_backend",
    "blockchain_items": [
        "block0.bin",
        "genesis.yaml"
    ]
}
```

### get files

User can list or view files available for current voting.
To list all available files `/api/control/files/list` endpoint can be utilized.
Then relative path can be provided in `/api/control/files/get/..` endpoint.\
For example: `http://{env_endpoint}:3030/api/control/files/get/qr-codes/zero_funds_12_0000.png`

- Request Type: GET
- Endpoint : <http://{env_endpoint}:3030/files/get/{file_path>}

## Interactive mode

TBD
