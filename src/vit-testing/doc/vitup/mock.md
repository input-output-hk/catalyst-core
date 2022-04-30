
## Mock

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

##### Make account endpoint unavailable

Mock will reject n calls to account endpoint and as a result voting app won't recieve voting power for some time.
This endpoint assume that one who changes block-account endpoint knows what is the frequency of calls from client
and ultimately can be translated to some time of unavailability.

```
curl --location --request POST 'http://{mock_address}/api/control/command/block-account/{number_of_calls_to_reject}'
```

##### Make account endpoint available

Mock will reset account endpoint unavailability 

```
curl --location --request POST 'http://{mock_address}/api/control/command/block-account/reset'
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