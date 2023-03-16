
# Mock

For developer convenience an in-memory backend is available.
Idea is the same as above but env is more lightweight and does not spawn jormungandr or vit-servicing-station.
Mock is also capable of controlling more backend aspect than normal deployment (cut off the connections, rejects all fragments.

## Configuration

Note: it is recommended to run command from `vit-testing/vitup` folder (then no explicit paths are required to be provided).
Configuration file example is available under `vit-testing/vitup/example/mock/config.yaml`

## Start

`vitup start mock --config example\mock\config.yaml`

### Admin rest commands

For postman collection please visit:

[Requests collection](../api/vitup/mock/mock_postman_collection.json)

#### List Files

```sh
curl --location --request GET 'http://{mock_address}/api/control/files/list'
```

#### Get File

```sh
curl --location --request GET 'http://{mock_address}/api/control/files/get/{path_to_file}'
```

#### Health

```sh
curl --location --request GET 'http://{mock_address}/api/health'
```

#### Change Fund Id

```sh
curl --location --request POST 'http://{mock_address}/api/control/command/fund/id/{new_fund_id}'
```

#### Add new fund

```sh
curl --location --request PUT 'http://{mock_address}/api/control/command/fund/update' \
--header 'Content-Type: application/json' \
--data-raw '
{
  "id": 20,
  "fund_name": "fund_3",
  "fund_goal": "How will we encourage developers and entrepreneurs to build Dapps and businesses on top of Cardano in the next 6 months?",
  "voting_power_threshold": 8000000000,
  "fund_start_time": "2022-05-04T10:50:41Z",
  "fund_end_time": "2022-05-04T11:00:41Z",
  "next_fund_start_time": "2022-06-03T10:40:41Z",
  "registration_snapshot_time": "2022-05-04T07:40:41Z",
  "next_registration_snapshot_time": "2022-06-02T10:40:41Z",
  "chain_vote_plans": [
    {
      "id": 2136640212,
      "chain_voteplan_id": "ad6eaebafd2cca7e1829df26c57b340a98b9d513b7eddec8561883f1b99f3b9e",
      "chain_vote_start_time": "2022-05-04T10:50:41Z",
      "chain_vote_end_time": "2022-05-04T11:00:41Z",
      "chain_committee_end_time": "2022-05-04T11:10:41Z",
      "chain_voteplan_payload": "public",
      "chain_vote_encryption_key": "",
      "fund_id": 20
    }
  ],
  "challenges": [
    {
      "id": 1,
      "challenge_type": "community-choice",
      "title": "Universal even-keeled installation",
      "description": "Upgradable",
      "rewards_total": 7686,
      "proposers_rewards": 844,
      "fund_id": 20,
      "challenge_url": "http://schneider-group.info",
      "highlights": {
        "sponsor": "Kreiger and Wuckert and Sons"
      }
    }
  ]
}

'
```

#### Accept all Fragments

Makes mock to accept all further fragments sent to environment

```sh
curl --location --request POST 'http://{mock_address}/api/control/command/fragments/accept'
```

#### Reject all Fragments

Makes mock to reject all further fragments sent to environment

```sh
curl --location --request POST 'http://{mock_address}/api/control/command/fragments/reject'
```

#### Hold all Fragments

Makes mock to hold  all further fragments sent to environment

```sh
curl --location --request POST 'http://{mock_address}/api/control/command/fragments/pending'
```

#### Reset Fragment strategy

Makes mock to validate all further fragments sent to environment

```sh
curl --location --request POST 'http://{mock_address}/api/control/command/fragments/reset'
```

#### Make backend unavailable

Mock will reject all connections (returns 500)

```sh
curl --location --request POST 'http://{mock_address}/api/control/command/available/false'
```

#### Make backend available

Mock will accept all connections

```sh
curl --location --request POST 'http://{mock_address}/api/control/command/available/true'
```

#### Make account endpoint unavailable

Mock will reject n calls to account endpoint and as a result voting app won't receive voting power for some time.
This endpoint assume that one who changes block-account endpoint knows what is the frequency of calls from client
and ultimately can be translated to some time of unavailability.

```sh
curl --location --request POST 'http://{mock_address}/api/control/command/block-account/{number_of_calls_to_reject}'
```

#### Make account endpoint available

Mock will reset account endpoint unavailability

```sh
curl --location --request POST 'http://{mock_address}/api/control/command/block-account/reset'
```

#### Add new voters snapshot for specific tag

Add (or overwrite) voters snapshot for this particular tag

```sh
curl --location --request POST 'http://{mock_address}/api/control/command/snapshot/add/{tag}' \
--header 'Content-Type: application/json' \
--data-raw '
  [{"voting_group":"direct","voting_key":"241799302733178aca5c0beaa7a43d054cafa36ca5f929edd46313d49e6a0fd5","voting_power":10131166116863755484},{"voting_group":"dreps","voting_key":"0e3fe9b3e4098759df6f7b44bd9b962a53e4b7b821d50bb72cbcdf1ff7f669f8","voting_power":9327154517439309883}]'
```

#### Create new voters snapshot for specific tag

Create snapshot json which can be uploaded to mock by using `../snapshot/add` command.
See [mock configuration](./configuration.md) for more details. Example:

```sh
curl --location --request POST 'http://{mock_address}/api/control/command/snapshot/create' \
--header 'Content-Type: application/json' \
--data-raw '{
    "tag": "daily",
    "content": [
    {
        "count": 2,
        "level": 5000
    },
    {
        "name": "darek",
        "funds": 100
    },
    {
        "key":"318947a91d109da7109feaf4625c0cc4e83fe1636ed19408e43a1dabed4090a3",
        "funds":300
    }
]
}'
```

#### Reset environment

Resets environment data

```sh
curl --location --request POST 'http://{mock_address}/api/control/command/reset' \
--header 'Content-Type: application/json' \
--data-raw '{
  "initials": {
    "block0": [
      {
        "above_threshold": 10,
        "pin": "1234"
      },
      {
        "name": "darek",
        "pin": "1234",
        "funds": 10000
      }
    ]
  },
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

see [data generation guide](../data_generation/reset.md) for more details

#### Control Health

Checks if mock is up

```sh
curl --location --request POST 'http://{mock_address}/api/control/health'
```

#### Logs

Mock stores record of each request send to it. This endpoint gets all logs from mock

```sh
curl --location --request POST 'http://{mock_address}/api/control/logs/get'
```

### Admin cli

Admin CLI is an alternative for all above calls, available under vitup project.

example:

```sh
vitup-cli --endpoint {mock} disruption control health
```
