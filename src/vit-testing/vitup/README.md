# vitup

Vitup is a cli project which is capable to bootstrap catalyst backend which can be excercised by various tools. 
Initial purpose is to provide simple localhost backend for catalyst voting app. 

## build

before building vitup all dependencies need to be installed.
- valgrind
- jormungandr
- vit-servicing-station

then in order to build vitup in main project folder run:
`cargo build`

and install:

`cargo install --path vitup`

## quick start

The simplest configuration is available by using command:

`vitup start quick`

default endpoint will be exposed at `0.0.0.0:80` all data dumped to `.\mock`

## Configuration

This section describe configuration file which can be passed as argument for or in some cases send to already running environments in order to restart them with new settings

### Initials

Allows to provide initial addresses/voters which addresses would be put in block0.
Supported syntax

##### above threshold 

Amount of wallets which receive more than value defined in `static_data.voting_power` parameter

Example: 

```
{
	"above_threshold":30,
    "pin":"1234"
}
```
Pin would be set globally for all 30 addresses 

##### below threshold 

Amount of wallets wich receive less than value defined in ``static_data.voting_power`` parameter 

Example: 

```
{
	"below_threshold":30,
    "pin":"1234"
}
```
Pin would be set globally for all 30 addresses 

##### around level 

Amount of wallets wich have funds around defined level

Example: 
```
{
	"count":30,
    "level":1000,
    "pin":"1234"
}
```

##### zero funds 

Amount of wallets wich won't have any funds in block0

Example: 
```
{
	"zero_funds":30,
    "pin":"1234"
}
```

##### named wallet 

Wallet with custom pin and arbitrary funds amount,

Example: 
```
      {
        "name":"darek",
        "funds":8000,
        "pin":"1111"         
      },
```

##### external wallet 

Wallet with address and pin. For users who already generated address outside vitup.

Example: 
```
      {
        "address":"ca1qknqa67aflzndy0rvkmxhd3gvccme5637qch53kfh0slzkfgv5nwyq4hxu4",
        "funds":8000     
      },
```

### vote plan

##### vote time

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
"slots_per_epoch":60,
```
See [jormungandr docs](https://input-output-hk.github.io/jormungandr/concepts/blockchain.html) for more information.

NOTE: `slot_duration` is defined in `blockchain` section of configuration file

##### private

If true, then voting is private otherwise public. This parameters basically controls if votes choices are encrypted or not.

###### representatives_vote_plan

TBD, currently not used


#### example

```
 "vote_plan": {
        "vote_time": {
            "vote_start": 13,
            "tally_start": 98,
            "tally_end": 140,
            "slots_per_epoch": 3600        
        },
        "private": true,
        "representatives_vote_plan": false
    },
  },
```

##### blockchain

Set of parameters which controls blochain related configuration.

See [jormungandr docs](https://input-output-hk.github.io/jormungandr/advanced/01_the_genesis_block.html) for more information.

##### slot_duration

Describes how frequent block are produces by network. Slot duration is expressed in seconds. Cannot be longer that 128.

##### block_content_max_size

Describes how big a single block can be. Larger blocks can hold more transactions which results in faster transactions processing, however it put more requirements on space and network throughput.

##### block0_time
Optional parameter which defines start time of block0. It is useful when one want to defined voting phases that ends and starts precisely in required time. Otherwise block0_time is equal to current time when running vitup

###### tx_max_expiry_epochs
Optional parameter which defines what is the maximum duration (expressed in epochs) of transacton timeout.
Usually it is equal to 1.

###### consensus_leader_ids

Allows to override randomly generated consensus leaders ids.
Useful when we have our own pregenerated leaders keys for nodes.

##### linear_fees

Transactions fees which defined cost of transaction or vote.
- constant - constant fee added to each transaction
- coefficient - coefficient of each transaction output
- certificate - cost of sending certificate.

`constant + transaction.output * coefficient + certificate`

Example:

```
  "linear_fees": {
    "constant": 1,
    "coefficient": 1,
    "certificate": 2
  },
```
Above configuration will result in:

For transaction with 1 input and 1 output
`1 +  1 * 1 + 0 = 2`

For vote
`1 + 0 * 1 + 2 = 3`

##### committees

Committee is a wallet that is capable of tallying voting results.
Thit setting allows to use predefined committe rather than generate random by vitup.

### data

Section describes static data used for voting. Mostly defines parameters for [servicing station](https://github.com/input-output-hk/vit-servicing-station)

##### options

Defines options available for voters. Should be expressed as coma-separated values. For example:

`options: "yes,no"`

##### snapshot_time

Data in [rfc3339](https://datatracker.ietf.org/doc/html/rfc3339#section-5.6) format. Defines snapshot datetime.

##### next_vote_start_time

Data in [rfc3339](https://datatracker.ietf.org/doc/html/rfc3339#section-5.6) format. Defines what is the date of next voting. This data will be shown to users after current voting will ends.

##### next_snapshot_time

Data in [rfc3339](https://datatracker.ietf.org/doc/html/rfc3339#section-5.6) format. Defines snapshot datetime. This data will be shown to users after current voting will ends.

##### proposals

Number of proposals available for voting

##### challenges

Number of challenges available for voting. Challenge is a container for proposals for the same domain

##### reviews

Number of reviews for proposals

##### voting_power

Threshold for voting participation, expressed in ADA

##### fund_name

Name of fund

##### fund_id

Id of the fund. This parameter also controls behavior of catalyst voting app. If it's changed between two funds, voting app will refresh it state.

### service

Service related settings

NOTE: this section is ignored when only generating data using vitup. 

##### version

Control version of backend. Manipulating this parameter we can tell voting app to force user to self-update application.
    
##### https

Controls protocol over which vitup is available for client


### Full Example:

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
    },
    {
      "zero_funds": 10,
      "pin": "1234"
    }
  ],
  "vote_plan": {
        "vote_time": {
            "vote_start": 13,
            "tally_start": 98,
            "tally_end": 140,
            "slots_per_epoch": 3600
        },
        "private": true
  },
  "blockchain": {
    "slot_duration": 4,
    "block_content_max_size": 20971520,
    "block0_time": "2022-01-19T05:00:00Z",
    "linear_fees": {
       "constant": 0,
       "coefficient": 0,
       "certificate": 0
    }
  },
  "data": {
    "options": "yes,no",
    "snapshot_time": "2022-01-06T11:00:00Z",
    "next_snapshot_time": "2022-04-07T11:00:00Z",
    "next_vote_start_time": "2022-04-11T11:00:00Z",
    "proposals": 936,
    "challenges": 25,
    "reviews": 5190,    
    "voting_power": 450,
    "fund_name": "Fund7",
    "fund_id": 6
  },
  "version":"3.6"
}
```

### Data generation 

TBD

### Configuration modes

In order to take out the burden of providing entire configuration vitup has two configuration modes:

- `quick` - which runs on defaults and allow user to override most important parameters using cli arguments:

`vitup start quick`

- `advanced` - which allows to defined full configuration as well as external static files for proposals and challenges

`vitup start advanced`

### Run Modes

There are 4 run modes available in vitup:
- `interactive` - where user can push some fragments or query status of nodes 
- `endless` - [Default] just simple run until stopped by user
- `service` - additional manager service will be published at `0.0.0.0:3030` and allow to control (stop/start/reset) and provides resources over http (qr codes or secret keys)
- `mock` - lightweight version of backend with does not spawn any jormungandr or vit-servicing-station services. Mock is also capable of controlling more backend aspect than normal deployment (cut off the connections, rejects all fragments.


### Endless mode

There are two ways of starting vitup in endless mode. One with limited configuration and one with giving full control. 

`vitup start quick --mode endless ..` or

`vitup start advanced --mode endless ..`


### Service mode

`vitup start quick --mode service ..` or

`vitup start advanced --mode service ..`

Once environment is up one can check status or modify existing environment:

#### Admin Operations
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

#### how to send operations

Voting backend admin console is an REST API accessible over http or https on port 3030. Using POST/GET http methods admin can send some operations to environment.
There are various apps capable of sending REST commands. The simplest is to download `Postman` and use UI to fire up commands.

1. Download postman: `https://www.postman.com/downloads/`
2. Review quick guide, how to send dummy request: `https://learning.postman.com/docs/getting-started/sending-the-first-request/`
3. Review guide, how to send different requests with arguments: `https://learning.postman.com/docs/sending-requests/requests/`

Available commands:

#### check environment status

- Request Type: GET
- Endpoint : http://{env_endpoint}:3030/status
- Response Example:
```
Running
```

#### start environment

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
- BODY: json with configuration
- Response Example:
```
start event received
```

This requests need to pass environment configuration file in Body.

#### stop environment

- Request Type: POST
- Endpoint : `http://{env_endpoint}:3030/api/control/command/stop`
- Response Example:
```
stop event received
```

#### list files
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


#### list files

User can list or view files available for current voting. to list all available files `/api/controlfiles/list` endpoint can be utilized. Then relative path can be provided in `/api/control/files/get/..` endpoint. For example:
`http://{env_endpoint}:3030/api/control/files/get/qr-codes/zero_funds_12_0000.png`


- Request Type: GET
- Endpoint : http://{env_endpoint}:3030/files/get/{file_path}

### Interactive mode

TBD


### Mock

For developer convience an in-memory backend is available. Idea is the same as above but env is more lightweight and does not spawn jormungandr or vit-servicing-station.
Mock is also capable of controlling more backend aspect than normal deployment (cut off the connections, rejects all fragments.

### Configuration

Note: it is recommended to run command from `vit-testing/vitup` folder (then no explicit paths are required to be provided).
Configuration file example is available under `vit-testing/vitup/example/mock/config.yaml`

### Start

`vitup start mock --config example\mock\config.yaml`

#### Admin rest commands

##### List Files

```
curl --location --request GET 'http://{mock_address}/api/control/files/list'
```

##### Get File

```
curl --location --request GET 'http://{mock_address}/api/control/files/get/{path_to_file}'
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
curl --location --request POST 'http://{mock_address}/api/control/command/reset' \
--header 'Content-Type: application/json' \
--data-raw '{ 
  "initials": [ 
    { 
      "above_threshold": 10,
      "pin": "1234"
    },
    {
      "name": "darek",
      "pin": "1234",
      "funds": 10000
    }
  ],
  "vote_plan": {
        "vote_time": {
            "vote_start": 0,
            "tally_start": 100,
            "tally_end": 140,
            "slots_per_epoch": 3600
        },
        "private": true
  },
  "blockchain": {
    "slot_duration": 2,
    "block_content_max_size": 20971520,
    "block0_time": "2022-03-17T05:00:00Z",
    "linear_fees": {
       "constant": 0,
       "coefficient": 0,
       "certificate": 0
    }
  },
  "data": {
    "options": "yes,no",
    "snapshot_time": "2022-01-06T11:00:00Z",
    "next_snapshot_time": "2022-04-07T11:00:00Z",
    "next_vote_start_time": "2022-04-11T11:00:00Z",
    "proposals": 936,
    "challenges": 25,
    "reviews": 5190,    
    "voting_power": 450,
    "fund_name": "Fund7",
    "fund_id": 6
  },
  "version":"3.6"
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