## Configuration

This section describe configuration file which can be passed as argument when starting vitup or in some cases send to already running environments in order to restart them with new settings

### Initials

#### snapshot 

Allows to provide initial voters and representatives which whose will be available in initial snapshot.

see [snapshot data creation guide](../snapshot.md) for more details

#### block0

Allows to provide initial addresses/voters which addresses would be put in block0.
Supported syntax:

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

Amount of wallets which receive less than value defined in ``static_data.voting_power`` parameter

Example:

```
{
	"below_threshold":30,
    "pin":"1234"
}
```
Pin would be set globally for all 30 addresses

##### around level

Amount of wallets which have funds around defined level

Example:
```
{
	"count":30,
    "level":1000,
    "pin":"1234"
}
```

##### zero funds

Amount of wallets which won't have any funds in block0

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

#### snapshot

Allows to provide initial addresses/voters which addresses would be put in initial snapshot.
Supported syntax:

##### random

Some number of random wallets which receive specified amount of voting power

Example:

```
  {
    "count": 2,
    "level": 5000
  },
```

##### external

A single entry with specified voting key and voting power

Example:

```
  {
    "key":"3877098d14e80c62c071a1d82e3df0eb9a6cd339a5f66e9ec338274fdcd9d0f4",
    "funds":300
  }
```

##### named

A single entry with specified alias from block0 and optional voting power. If voting power is not defined it would be taken from block0 section. If vitup cannot find alias it will produce an error

Example:

```
  {
    "name": "darek",
    "funds": 100
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

Set of parameters which controls blockchain related configuration.

See [jormungandr docs](https://input-output-hk.github.io/jormungandr/advanced/01_the_genesis_block.html) for more information.

##### slot_duration

Describes how frequent block are produces by network. Slot duration is expressed in seconds. Cannot be longer that 128.

##### block_content_max_size

Describes how big a single block can be. Larger blocks can hold more transactions which results in faster transactions processing, however it put more requirements on space and network throughput.

##### block0_time
Optional parameter which defines start time of block0. It is useful when one want to defined voting phases that ends and starts precisely in required time. Otherwise block0_time is equal to current time when running vitup

###### tx_max_expiry_epochs
Optional parameter which defines what is the maximum duration (expressed in epochs) of transaction timeout.
Usually it is equal to 1.

###### consensus_leader_ids

Allows to override randomly generated consensus leaders ids.
Useful when we have our own pre-generated leaders keys for nodes.

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
This setting allows to use predefined committee rather than generate random by vitup.

### data

Section describes static data used for voting. Mostly defines parameters for [servicing station](https://github.com/input-output-hk/vit-servicing-station)


#### current fund

Current fund related settings:


##### options

Defines options available for voters. Should be expressed as coma-separated values. For example:

`options: "yes,no"`

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

##### dates

###### proposal_submission_start

Data in [rfc3339](https://datatracker.ietf.org/doc/html/rfc3339#section-5.6) format. Defines proposal submission start datetime.

###### insight_sharing_start

Data in [rfc3339](https://datatracker.ietf.org/doc/html/rfc3339#section-5.6) format. Defines proposal insight sharing start datetime.

###### refine_proposals_start

Data in [rfc3339](https://datatracker.ietf.org/doc/html/rfc3339#section-5.6) format. Defines proposal refinement start datetime.

###### finalize_proposals_start

Data in [rfc3339](https://datatracker.ietf.org/doc/html/rfc3339#section-5.6) format. Defines proposal finalization start datetime.

###### proposal_assessment_start

Data in [rfc3339](https://datatracker.ietf.org/doc/html/rfc3339#section-5.6) format. Defines proposal assessment start datetime.

###### assessment_qa_start

Data in [rfc3339](https://datatracker.ietf.org/doc/html/rfc3339#section-5.6) format. Defines proposal assessment qa start datetime.

###### snapshot_time

Data in [rfc3339](https://datatracker.ietf.org/doc/html/rfc3339#section-5.6) format. Defines snapshot datetime.

###### next_vote_start_time

Data in [rfc3339](https://datatracker.ietf.org/doc/html/rfc3339#section-5.6) format. Defines what is the date of next voting. This data will be shown to users after current voting will ends.

###### next_snapshot_time

Data in [rfc3339](https://datatracker.ietf.org/doc/html/rfc3339#section-5.6) format. Defines next snapshot datetime. This data will be shown to users after current voting will ends.

#### next funds

Limited subset of settings comparing to `current_fund` section for next funds

##### fund_name

Name of fund

##### fund_id

Id of the fund. This parameter also controls behavior of catalyst voting app. If it's changed between two funds, voting app will refresh it state.

##### dates

###### proposal_submission_start

Data in [rfc3339](https://datatracker.ietf.org/doc/html/rfc3339#section-5.6) format. Defines proposal submission start datetime.

###### insight_sharing_start

Data in [rfc3339](https://datatracker.ietf.org/doc/html/rfc3339#section-5.6) format. Defines proposal insight sharing start datetime.

###### refine_proposals_start

Data in [rfc3339](https://datatracker.ietf.org/doc/html/rfc3339#section-5.6) format. Defines proposal refinement start datetime.

###### finalize_proposals_start

Data in [rfc3339](https://datatracker.ietf.org/doc/html/rfc3339#section-5.6) format. Defines proposal finalization start datetime.

###### proposal_assessment_start

Data in [rfc3339](https://datatracker.ietf.org/doc/html/rfc3339#section-5.6) format. Defines proposal assessment start datetime.

###### assessment_qa_start

Data in [rfc3339](https://datatracker.ietf.org/doc/html/rfc3339#section-5.6) format. Defines proposal assessment qa start datetime.

###### snapshot_time

Data in [rfc3339](https://datatracker.ietf.org/doc/html/rfc3339#section-5.6) format. Defines snapshot datetime.

###### next_vote_start_time

Data in [rfc3339](https://datatracker.ietf.org/doc/html/rfc3339#section-5.6) format. Defines what is the date of next voting. This data will be shown to users after current voting will ends.

###### next_snapshot_time

Data in [rfc3339](https://datatracker.ietf.org/doc/html/rfc3339#section-5.6) format. Defines next snapshot datetime.

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
   "initials":{
      "snapshot":{
         "tag":"daily",
         "content":[
            {
               "count":2,
               "level":1234
            },
            {
               "name":"alice"
            },
            {
               "name":"bob",
               "funds":10001
            }
         ]
      },
      "block0":[
         {
            "above_threshold":10,
            "pin":"1234"
         },
         {
            "name":"alice",
            "pin":"1234",
            "funds":10000
         },
         {
            "name":"bob",
            "pin":"1234",
            "funds":10000
         },
         {
            "zero_funds":10,
            "pin":"1234"
         }
      ]
   },
   "vote_plan":{
      "vote_time":{
         "vote_start":0,
         "tally_start":134,
         "tally_end":234,
         "slots_per_epoch":3600
      },
      "private":true
   },
   "blockchain":{
      "slot_duration":4,
      "block_content_max_size":20971520,
      "linear_fees":{
         "constant":0,
         "coefficient":0,
         "certificate":0
      }
   },
   "data":{
      "current_fund":{
         "options":"yes,no",
         "proposals":1134,
         "challenges":23,
         "reviews":7045,
         "voting_power":450,
         "fund_name":"Fund9",
         "fund_id":9,
         "dates":{
            "insight_sharing_start":"2022-05-01T12:00:00Z",
            "proposal_submission_start":"2022-05-02T12:00:00Z",
            "refine_proposals_start":"2022-05-03T12:00:00Z",
            "finalize_proposals_start":"2022-05-04T12:00:00Z",
            "proposal_assessment_start":"2022-05-04T12:00:00Z",
            "assessment_qa_start":"2022-05-05T12:00:00Z",
            "snapshot_time":"2022-05-07T12:00:00Z",
            "next_snapshot_time":"2023-05-07T12:00:00Z",
            "next_vote_start_time":"2022-07-14T12:00:00Z"
         }
      },
      "next_funds":[
         {
            "fund_name":"Fund10",
            "fund_id":10,
            "dates":{
               "insight_sharing_start":"2023-05-01T12:00:00Z",
               "proposal_submission_start":"2023-05-02T12:00:00Z",
               "refine_proposals_start":"2023-05-03T12:00:00Z",
               "finalize_proposals_start":"2023-05-04T12:00:00Z",
               "proposal_assessment_start":"2023-05-04T12:00:00Z",
               "assessment_qa_start":"2023-05-05T12:00:00Z",
               "snapshot_time":"2023-05-07T12:00:00Z",
               "voting_start":"2023-07-14T12:00:00Z",
               "voting_tally_end":"2023-07-14T12:00:00Z",
               "voting_tally_start":"2023-07-14T12:00:00Z",
               "next_snapshot_time":"2023-07-07T12:00:00Z",
               "next_vote_start_time":"2023-07-14T12:00:00Z"
            }
         }
      ]
   },
   "version":"3.8"
}
```
