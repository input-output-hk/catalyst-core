# vitup

Vitup is a cli project which is capable to bootstrap mock backend which can be excercised by various tools. Initial purpose is to provide simple localhost backend for 
catalyst voting app

## build

before building vitup all dependencies need to be installed.
- iapyx-proxy
- jormungandr
- vit-servicing-station

then in order to build vitup:
`cargo build`

and install:

`cargo install --path .`

## quick start

The simplest configuration is available by using command:

`vitup start quick`

default endpoint will be exposed at `0.0.0.0:80` all data dumped to `.\data\vit_backend`

### Command line parameters

#### Modes

There are 3 modes available in vitup:
- `--mode interactive` - where user can push some fragments or query status of nodes 
- `--mode endless` - [Default] just simple run until stopped by user
- `--mode service` - manager service published at `0.0.0.0:3030` and control stop/start/ and provide files over http


##### Admin

Once environment is up admin can check status or modify existing environment. It can use below operations


###### Admin Operations
- start -  in order to start new voting
- stop -  stops currently running vote backend (usually it takes 1 min to stop it)
- status -  check status of environment: <br/>
  a) `Idle` - environment is not started, <br/>
  b) `Starting` - environment is starting, please wait until its status is `Running`, <br/>
  c) `Running` - environment is not running and should be accessible, <br/>
  d) `Stopping` - environment is stopping, please wait until its `Idle` to start it with different parameters,
  
- files: 
In order to get qr-codes or secret files from env, two operations are provided: <br/>
  a) `List Files` - list all files in data directory for current run, <br/>
  b) `Get File` - downloads particular file which is visible in `List Files` operation result,

###### How to send operations

Voting backend admin console is an REST API accessible over http on port 3030. Using POST/GET http methods admin can send some operations to environment.
There are various apps capable of sending REST commands. The simplest is to download `Postman` and use UI to fire up commands.

1. Download postman: `https://www.postman.com/downloads/`
2. Review quick guide, how to send dummy request: `https://learning.postman.com/docs/getting-started/sending-the-first-request/`
3. Review guide, how to send different requests with arguments: `https://learning.postman.com/docs/sending-requests/requests/`

Now let's review available commands:

###### Check environment status

- Request Type: GET
- Endpoint : http://{env_endpoint}:3030/status
- Response Example:
```
Running
```

###### Start environment

Default parameters:

- Request Type: POST
- Endpoint : http://{env_endpoint}:3030/api/status
- Response Example:
```
start event received
```

Custom parameters:

- Request Type: POST
- Endpoint : http://{env_endpoint}:3030/api/control/command/start/default
- BODY: json
- Response Example:
```
start event received
```

This requests need to pass environment configuration file in Body.

###### Configuration file

This section describe configuration file which can be passed to `api/control/command/start/custom` endpoint

- Initials:


Allows to provide initial addresses/voters which addresses would be put in block0.
Supported syntax
a) above_threshold - amount of wallets which receive more than value defined in `threshold` parameter

Example: 

```
{
	"above_threshold":30,
    "pin":"1234"
}
```
Pin would be set globally for all 30 addresses 


b) below_threshold - amount of wallets wich receive less than value defined in `threshold` parameter 

Example: 

```
{
	"below_threshold":30,
    "pin":"1234"
}
```
Pin would be set globally for all 30 addresses 

c) zero_funds - amount of wallets wich won't have any funds in block0

Example: 
```
{
	"zero_funds":30,
    "pin":"1234"
}
```

d) named wallet - you can add alias to wallet with custom pin and arbitrary funds amount,

Example: 
```
      {
        "name":"darek",
        "funds":8000,
        "pin":"1111"         
      },
```

- Vote phases timing 
Below parameters describe how long vote would be active, for how long users can vote and when tally period would begin.


In cardano time is divided into epochs which consists of slots. There are 2 parameters that defines how long epoch should last, slot_duration and slots_per_epoch with equation:
`epoch_duration = slot_duration * slots_per_epoch`.

For example, for given:
```
slot_duration = 2
slots_per_epoch = 10
```

then epoch will lasts 20 seconds.

vote_start, vote_tally, tally_end - describe 2 vote phases: 
- from vote_start to vote_tally : casting vote period, where we gather votes.
- from vote_tally to tally_end: tallying vote period, where we gather voting results.

Sll above parameters are expressed in epochs. Be aware that `slot_duration` and `slots_per_epoch` have influence on time voting phase would start.
For example if we would like to start vote in 5 minutes, allow users to case vote for 20 minutes and give 1 hour for tally operation our setup would be like below:
```
"vote_start":1,
"vote_tally":4,
"tally_end":20,
"slot_duration":5,
"slots_per_epoch":60,
```

- Proposals: Number of proposals available for voting

- Voting power:  Threshold for voting participation, expressed in ADA

- Fund name:  Name of fund

- Private:  If true, then voting is private otherwise public

- Secure endpoint: Controls if mock will be exposed as http or https

Full Example:

```
{  
  "initials": [
    {
      "above_threshold": 10,
      "pin": "1234"
    },
    {
      "name": "alice",
      "pin": "1234",
      "funds": 10000
    },
    {
      "name": "bob",
      "pin": "1234",
      "funds": 10000
    }
  ],
  "vote_time": {
    "vote_start": 5,
    "tally_start": 100,
    "tally_end": 200,
    "slots_per_epoch": 900
  },
  "snapshot_time": "2021-09-30T09:00:00",
  "next_vote_start_time": "2021-10-07T21:00:00",
  "proposals": 267,
  "challenges": 9,
  "reviews": 5072,
  "slot_duration": 5,
  "voting_power": 450,
  "fund_name": "Fund6",
  "block_content_max_size": 204800,
  "private": true,
  "fund_id":6,
  "linear_fees": {
    "constant": 0,
    "coefficient": 0,
    "certificate": 0
  },
  "version":"2.0"
}
```

###### stop environment

- Request Type: POST
- Endpoint : `http://{env_endpoint}:3030/api/control/command/stop`
- Response Example:
```
stop event received
```

###### list files
- Request Type: GET
- Endpoint : `http://{env_endpoint}:3030/api/control/files/list`
- Response Example:
```
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


###### list files

User can list or view files available for current voting. to list all available files `/api/controlfiles/list` endpoint can be utilized. Then relative path can be provided in `/api/control/files/get/..` endpoint. For example:
`http://{env_endpoint}:3030/api/control/files/get/qr-codes/zero_funds_12_0000.png`


- Request Type: GET
- Endpoint : http://{env_endpoint}:3030/files/get/{file_path}


## Mock

For developer convience an in-memory backend is available. Idea is the same as above but env is more lightweight and does not spawn jormungandr or vit-servicing-station.
Mock is also capable of controlling more backend aspect than normal deployment (cut off the connections, rejects all fragments.

### Configuration

Note: it is recommended to run command from `vit-testing/vitup` folder (then no explicit paths are required to be provided).
Configuration file example is available under `vit-testing/vitup/config.yaml`

### Start

`vitup start mock --config example\mock\config.yaml`

#### Admin rest commands

##### List Files

```
curl --location --request GET 'http://{mock_address}/api/control/files/list'
```

##### Health


```
curl --location --request GET 'http://{mock_address}/api/health'
```

##### Change Fund Id

```
curl --location --request POST 'http://{mock_address}/api/control/command/fund/id/{new_fund_id}'
```

##### Accept all Fragments

Makes mock to accept all further fragments sent to environment

```
curl --location --request POST 'http://{mock_address}/api/control/command/fragments/accept'
```

##### Reject all Fragments

Makes mock to reject all further fragments sent to environment

```
curl --location --request POST 'http://{mock_address}/api/control/command/fragments/reject'
```

##### Hold all Fragments


Makes mock to hold  all further fragments sent to environment

```
curl --location --request POST 'http://{mock_address}/api/control/command/fragments/pending'
```

##### Reset Fragment strategy


Makes mock to validate all further fragments sent to environment

```
curl --location --request POST 'http://{mock_address}/api/control/command/fragments/reset'
```
##### Make backend unavailable

Mock will reject all connections (returns 500)

```
curl --location --request POST 'http://{mock_address}/api/control/command/available/false'
```

##### Make backend available


Mock will accept all connections

```
curl --location --request POST 'http://{mock_address}/api/control/command/available/true'
```
##### Reset environment

Resets environment data

```
curl --location --request POST 'http://192.168.0.19:8080/api/control/command/reset' \
--header 'Content-Type: application/json' \
--data-raw '{ 
  "initials": [ 
    { 
      "above_threshold": 10,
      "pin": "1234"
    },
    {
      "name": "dor",
      "pin": "1234",
      "funds": 10000
    },
    {
      "name": "eric",
      "pin": "1234",
      "funds": 10000
    },
    {
      "name": "kriss",
      "pin": "1234",
      "funds": 10000
    },
    {
      "name": "marek",
      "pin": "1234",
      "funds": 10000
    },
    {
      "name": "darek",
      "pin": "1234",
      "funds": 10000
    }
  ],
  "vote_time": {
    "vote_start": 5,
    "tally_start": 100,
    "tally_end": 200,
    "slots_per_epoch": 900
  },
  "snapshot_time": "2021-08-12T09:00:00",
  "next_vote_start_time": "2021-10-07T21:00:00",
  "proposals": 267,
  "challenges": 9,
  "reviews": 5072,
  "slot_duration": 5,
  "voting_power": 450,
  "fund_name": "Fund6",
  "block_content_max_size": 204800,
  "private": true,
  "fund_id":6,
  "linear_fees": {
    "constant": 0,
    "coefficient": 0,
    "certificate": 0
  },
  "version":"2.0"
}'
```
##### Health

Checks if mock is up

```
curl --location --request POST 'http://{mock_address}/api/control/health'
```

##### Logs

Mock stores record of each request send to it. This endpoint gets all logs from mock


```
curl --location --request POST 'http://{mock_address}/api/control/logs/get'
```


#### Admin cli


Admin CLI is an alternative for all above calls, available under vitup project.

example:

```
vitup-cli --endpoint {mock} disruption control health
```
