# REST

Jormungandr comes with a CLI client for manual communication with nodes over HTTP.

## Conventions

Many CLI commands have common arguments:

- `-h <addr>` or `--host <addr>` - Node API address. Must always have `http://` or
`https://` prefix. E.g. `-h http://127.0.0.1`, `--host https://node.com:8443/cardano/api`
- `--debug` - Print additional debug information to stderr.
The output format is intentionally undocumented and unstable
- `--output-format <format>` - Format of output data. Possible values: json, yaml, default yaml.
Any other value is treated as a custom format using values from output data structure.
Syntax is Go text template: https://golang.org/pkg/text/template/.

## Node stats

Fetches node stats

```sh
jcli rest v0 node stats get <options>
```

The options are

- -h <node_addr> - see [conventions](#conventions)
- --debug - see [conventions](#conventions)
- --output-format \<format\> - see [conventions](#conventions)

YAML printed on success

```yaml
---
# Number of blocks received by node
blockRecvCnt: 1102
# Size in bytes of all transactions in last block
lastBlockContentSize: 484
# The Epoch and slot Number of the block (optional)
lastBlockDate: "20.29"
# Sum of all fee values in all transactions in last block
lastBlockFees: 534
# The block hash, it's unique identifier in the blockchain (optional)
lastBlockHash: b9597b45a402451540e6aabb58f2ee4d65c67953b338e04c52c00aa0886bd1f0
# The block number, in order, since the block0 (optional)
lastBlockHeight: 202901
# Sum of all input values in all transactions in last block
lastBlockSum: 51604
# The time slot of the tip block
lastBlockTime: "2020-01-30T22:37:46+00:00"
# Number of transactions in last block
lastBlockTx: 2
# The time at which we received the last block, not necessarily the current tip block (optional)
lastReceivedBlockTime: "2020-01-30T22:37:59+00:00"
# 24 bytes encoded in hexadecimal Node ID
nodeId: "ad24537cb009bedaebae3d247fecee9e14c57fe942e9bb0d"
# Number of nodes that are available for p2p discovery and events propagation
peerAvailableCnt: 321
# Number of nodes that have been quarantined by our node
peerQuarantinedCnt: 123
# Total number of nodes
peerTotalCnt: 444
# Number of nodes that are connected to ours but that are not publicly reachable
peerUnreachableCnt: 0
# State of the node
state: Running
# Number of transactions received by node
txRecvCnt: 5440
# Node uptime in seconds
uptime: 20032
# Node app version
version: jormungandr 0.8.9-30d20d2e
```

## Get UTxO

Fetches UTxO details

```sh
jcli rest v0 utxo <fragment-id> <output-index> get <options>
```

- \<fragment-id\>   - hex-encoded ID of the transaction fragment
- \<output-index\>  - index of the transaction output

The options are

- -h <node_addr> - see [conventions](#conventions)
- --debug - see [conventions](#conventions)
- --output-format \<format\> - see [conventions](#conventions)

YAML printed on success

```yaml
---
# UTxO owner address
address: ca1svs0mwkfky9htpam576mc93mee5709khre8dgnqslj6y3p5f77s5gpgv02w
# UTxO value
value: 10000
```

## Post transaction

Posts a signed, hex-encoded transaction

```sh
jcli rest v0 message post <options>
```

The options are

- -h <node_addr> - see [conventions](#conventions)
- --debug - see [conventions](#conventions)
- -f --file <file_path> - File containing hex-encoded transaction.
If not provided, transaction will be read from stdin.

Fragment Id is printed on success (which can help finding transaction status using get message log command)

```sh
50f21ac6bd3f57f231c4bf9c5fff7c45e2529c4dffed68f92410dbf7647541f1
```

## Get message log

Get the node's logs on the message pool. This will provide information on pending transaction,
rejected transaction and or when a transaction has been added in a block

```sh
jcli rest v0 message logs <options>
```

The options are

- -h <node_addr> - see [conventions](#conventions)
- --debug - see [conventions](#conventions)
- --output-format \<format\> - see [conventions](#conventions)

YAML printed on success

```yaml
---
- fragment_id: 7db6f91f3c92c0aef7b3dd497e9ea275229d2ab4dba6a1b30ce6b32db9c9c3b2 # hex-encoded fragment ID
  last_updated_at: 2019-06-02T16:20:26.201000000Z                               # RFC3339 timestamp of last fragment status change
  received_at: 2019-06-02T16:20:26.201000000Z                                   # RFC3339 timestamp of fragment receivement
  received_from: Network,                                                       # how fragment was received
  status: Pending,                                                              # fragment status
```

`received_from` can be one of:

```yaml
received_from: Rest     # fragment was received from node's REST API
```

```yaml
received_from: Network  # fragment was received from the network
```

`status` can be one of:

```yaml
status: Pending                 # fragment is pending
```

```yaml
status:
  Rejected:                     # fragment was rejected
    reason: reason of rejection # cause
```

```yaml
status:                         # fragment was included in a block
  InABlock:
    date: "6637.3"            # block epoch and slot ID formed as <epoch>.<slot_id>
    block: "d9040ca57e513a36ecd3bb54207dfcd10682200929cad6ada46b521417964174"
```

## Blockchain tip

Retrieves a hex-encoded ID of the blockchain tip

```sh
jcli rest v0 tip get <options>
```

The options are

- -h <node_addr> - see [conventions](#conventions)
- --debug - see [conventions](#conventions)

## Get block

Retrieves a hex-encoded block with given ID

```sh
jcli rest v0 block <block_id> get <options>
```

- <block_id> - hex-encoded block ID

The options are

- -h <node_addr> - see [conventions](#conventions)
- --debug - see [conventions](#conventions)

## Get next block ID

Retrieves a list of hex-encoded IDs of descendants of block with given ID.
Every list element is in separate line. The IDs are sorted from closest to farthest.

```sh
jcli rest v0 block <block_id> next-id get <options>
```

- <block_id> - hex-encoded block ID

The options are

- -h <node_addr> - see [conventions](#conventions)
- --debug - see [conventions](#conventions)
- -c --count \<count\> - Maximum number of IDs, must be between 1 and 100, default 1

## Get account state

Get account state

```sh
jcli rest v0 account get <account-id> <options>
```

- \<account-id\> - ID of an account, bech32-encoded

The options are

- -h <node_addr> - see [conventions](#conventions)
- --debug - see [conventions](#conventions)
- --output-format \<format\> - see [conventions](#conventions)

YAML printed on success

```yaml
---
counter: 1
delegation: c780f14f9782770014d8bcd514b1bc664653d15f73a7158254730c6e1aa9f356
value: 990
```

- `value` is the current balance of the account;
- `counter` is the number of transactions performed using this account
  this is useful to know when signing new transactions;
- `delegation` is the Stake Pool Identifier the account is delegating to.
  it is possible this value is not set if there is no delegation certificate
  sent associated to this account.

## Node settings

Fetches node settings

```sh
jcli rest v0 settings get <options>
```

The options are

- -h <node_addr> - see [conventions](#conventions)
- --debug - see [conventions](#conventions)
- --output-format \<format\> - see [conventions](#conventions)

YAML printed on success

```yaml
---
block0Hash: 8d94ecfcc9a566f492e6335858db645691f628b012bed4ac2b1338b5690355a7  # block 0 hash of
block0Time: "2019-07-09T12:32:51+00:00"         # block 0 creation time of
blockContentMaxSize: 102400                     # the block content's max size in bytes
consensusVersion: bft                           # currently used consensus
currSlotStartTime: "2019-07-09T12:55:11+00:00"  # current slot start time
epochStabilityDepth: 102400                     # the depth, number of blocks, to which we consider the blockchain to be stable and prevent rollback beyond that depth
fees:                                           # transaction fee configuration
  certificate: 4                                # fee per certificate
  coefficient: 1                                # fee per every input and output
  constant: 2                                   # fee per transaction
  per_certificate_fees:                         # fee per certificate operations, all zero if this object absent (optional)
    certificate_pool_registration: 5            # fee per pool registration, zero if absent (optional)
    certificate_stake_delegation: 15            # fee per stake delegation, zero if absent (optional)
    certificate_owner_stake_delegation: 2       # fee per pool owner stake delegation, zero if absent (optional)
rewardParams:                                   # parameters for rewards calculation
  compoundingRatio:                             # speed at which reward is reduced. Expressed as numerator/denominator
    denominator: 1024
    numerator: 1
  compoundingType: Linear                       # reward reduction algorithm. Possible values: "Linear" and "Halvening"
  epochRate: 100                                # number of epochs between reward reductions
  epochStart: 0                                 # epoch when rewarding starts
  initialValue: 10000                           # initial reward
slotDuration: 5                                 # slot duration in seconds
slotsPerEpoch: 720                              # number of slots per epoch
treasuryTax:                                    # tax from reward that goes to pot
  fixed: 5                                      # what get subtracted as fixed value
  ratio:                                        # ratio of tax after fixed amount is subtracted. Expressed as numerator/denominator
    numerator: 1
    denominator: 10000
  max: 100                                      # limit of tax (optional)
```

## Node shutdown

Node shutdown

```sh
jcli rest v0 shutdown get <options>
```

The options are

- -h <node_addr> - see [conventions](#conventions)
- --debug - see [conventions](#conventions)

## Get leaders

Fetches list of leader IDs

```sh
jcli rest v0 leaders get <options>
```

The options are

- -h <node_addr> - see [conventions](#conventions)
- --debug - see [conventions](#conventions)
- --output-format \<format\> - see [conventions](#conventions)

YAML printed on success

```yaml
---
- 1 # list of leader IDs
- 2
```

## Register leader

Register new leader and get its ID

```sh
jcli rest v0 leaders post <options>
```

The options are

- -h <node_addr> - see [conventions](#conventions)
- --debug - see [conventions](#conventions)
- --output-format \<format\> - see [conventions](#conventions)
-f, --file \<file\> - File containing YAML with leader secret. It must have the same format as secret YAML passed to Jormungandr as --secret. If not provided, YAML will be read from stdin.

On success created leader ID is printed

```sh
3
```

## Delete leader

Delete leader with given ID

```sh
jcli rest v0 leaders delete <id> <options>
```

- \<id\> - ID of deleted leader

The options are

- -h <node_addr> - see [conventions](#conventions)
- --debug - see [conventions](#conventions)

## Get leadership logs

Fetches leadership logs

```sh
jcli rest v0 leaders logs get <options>
```

The options are

- -h <node_addr> - see [conventions](#conventions)
- --debug - see [conventions](#conventions)
- --output-format \<format\> - see [conventions](#conventions)

YAML printed on success

```yaml
---
- created_at_time: "2019-08-19T12:25:00.417263555+00:00"
  enclave_leader_id: 1
  finished_at_time: "2019-08-19T23:19:05.010113333+00:00"
  scheduled_at_date: "0.3923"
  scheduled_at_time: "2019-08-19T23:18:35+00:00"
  wake_at_time: "2019-08-19T23:18:35.001254555+00:00"
  status:
    Block:
      chain_length: 201018
      block: d9040ca57e513a36ecd3bb54207dfcd10682200929cad6ada46b521417964174
      parent: cc72d4ca957b03d7c795596b7fd7b1ff09c649c3e2877c508c0466abc8604832

```

Different value for the status:

```yaml
# meaning the action is still pending to happen
status: Pending
```

```yaml
# meaning the action successfully create the given block with the given hash and parent
status:
  Block:
    chain_length: 201018
    block: d9040ca57e513a36ecd3bb54207dfcd10682200929cad6ada46b521417964174
    parent: cc72d4ca957b03d7c795596b7fd7b1ff09c649c3e2877c508c0466abc8604832
```

```yaml
# meaning the event has failed for some reasons
status:
  Rejected:
    reason: "Missed the deadline to compute the schedule"
```

## Get stake pools

Fetches list of stake pool IDs

```sh
jcli rest v0 stake-pools get <options>
```

The options are

- -h <node_addr> - see [conventions](#conventions)
- --debug - see [conventions](#conventions)
- --output-format \<format\> - see [conventions](#conventions)

YAML printed on success

```yaml
---
- 5cf03f333f37eb7b987dbc9017b8a928287a3d77d086cd93cd9ad05bcba7e60f # list of stake pool IDs
- 3815602c096fcbb91072f419c296c3dfe1f730e0f446a9bd2553145688e75615
```

## Get stake distribution

Fetches stake information

```sh
jcli rest v0 stake get <options> [<epoch>]
```

- \<epoch\> - Epoch to get the stake distribution from. (optional)

The options are

- -h <node_addr> - see [conventions](#conventions)
- --debug - see [conventions](#conventions)
- --output-format \<format\> - see [conventions](#conventions)

YAML printed on success

- `jcli rest v0 stake get <options>` - stake distribution from the current epoch

```yaml
---
epoch: 228      # Epoch of last block
stake:
  dangling: 0 # Total value stored in accounts, but assigned to nonexistent pools
  pools:
    - - 5cf03f333f37eb7b987dbc9017b8a928287a3d77d086cd93cd9ad05bcba7e60f # stake pool ID
      - 1000000000000                                                    # staked value
    - - 3815602c096fcbb91072f419c296c3dfe1f730e0f446a9bd2553145688e75615 # stake pool ID
      - 1000000000000                                                    # staked value
  unassigned: 0 # Total value stored in accounts, but not assigned to any pool
```

- `jcli rest v0 stake get <options> 10` - stake distribution from a specific epoch (epoch 10 in this example)

```yaml
---
epoch: 10      # Epoch specified in the request
stake:
  dangling: 0 # Total value stored in accounts, but assigned to nonexistent pools
  pools:
    - - 5cf03f333f37eb7b987dbc9017b8a928287a3d77d086cd93cd9ad05bcba7e60f # stake pool ID
      - 1000000000000                                                    # staked value
    - - 3815602c096fcbb91072f419c296c3dfe1f730e0f446a9bd2553145688e75615 # stake pool ID
      - 1000000000000                                                    # staked value
  unassigned: 0 # Total value stored in accounts, but not assigned to any pool
```

## Network stats

Fetches network stats

```sh
jcli rest v0 network stats get <options>
```

The options are

- -h <node_addr> - see [conventions](#conventions)
- --debug - see [conventions](#conventions)
- --output-format \<format\> - see [conventions](#conventions)

YAML printed on success

```yaml
---
- # node address (optional)
  addr: "3.124.55.91:3000"
  # hex-encoded node ID
  nodeId: 0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20
  # timestamp of when the connection was established
  establishedAt: "2019-10-14T06:24:12.010231281+00:00"
  # timestamp of last time block was received from node if ever (optional)
  lastBlockReceived: "2019-10-14T00:45:57.419496113+00:00"
  # timestamp of last time fragment was received from node if ever (optional)
  lastFragmentReceived: "2019-10-14T00:45:58.419496150+00:00"
  # timestamp of last time gossip was received from node if ever (optional)
  lastGossipReceived: "2019-10-14T00:45:59.419496188+00:00"
```

## Get stake pool details

Fetches stake pool details

```sh
jcli rest v0 stake-pool get <pool-id> <options>
```

- \<pool-id\> - hex-encoded pool ID

The options are

- -h <node_addr> - see [conventions](#conventions)
- --debug - see [conventions](#conventions)
- --output-format \<format\> - see [conventions](#conventions)

YAML printed on success

```yaml
---
tax:                        # pool reward
  fixed: 5                  # what get subtracted as fixed value
  ratio:                    # ratio of tax after fixed amount is subtracted. Expressed as numerator/denominator
    numerator: 1
    denominator: 10000
  max: 100                  # limit of tax (optional)
total_stake: 2000000000000  # total stake pool value
# bech32-encoded stake pool KES key
kesPublicKey: kes25519-12-pk1q7susucqwje0lpetqzjgzncgcrjzx7e2guh900qszdjskkeyqpusf3p39r
# bech32-encoded stake pool VRF key
vrfPublicKey: vrf_pk1rcm4qm3q9dtwq22x9a4avnan7a3k987zvepuxwekzj3uyu6a8v0s6sdy0l
```

## Get rewards history for a specific epoch

Get the rewards history of a given *epoch*.

```sh
jcli rest v0 rewards epoch get <epoch> <options>
```

- \<epoch\> - epoch number to get the rewards history for.

The options are

- -h <node_addr> - see [conventions](#conventions)
- --debug - see [conventions](#conventions)

```sh
jcli rest v0 rewards epoch get 82 -h <node_addr>
```

```json
[
  {
    "epoch": 82,              // the epoch number to collect rewards info from (rewards are from epoch 81)
    "drawn": 3835616440000,   // Total Drawn from reward escrow pot for the epoch
    "fees": 1828810000,       // Fees contributed into the pot the epoch
    "treasury": 462179124139, // Value added to the treasury
    "stake_pools": {
      "0087011b9c626759f19d9d0315a9b42492ba497438c12efc026d664c9f324ecb": [
        1683091391, // pool's owned rewards from taxes
        32665712521 // distributed rewards to delegators
      ],
      "014bb0d84f40900f6dd85835395bc38da3ab81435d1e6ee27d419d6eeaf7d16a": [
        47706672,
        906426770
      ],
    },
    "accounts": {
      "ed25519_pk1qqq6r7r7medu2kdpvdra5kwh8uz9frvftm9lf25shm7ygx9ayvss0nqke9": 427549785, // Amount added to each account
      "ed25519_pk1qqymlwehsztpzhy2k4szkp7j0xk0ra35jyxcpgr9p9q4ngvzzc5q4sh2gm": 24399360,
      "ed25519_pk1qq9h62jv6a0mz36xgecjrz9tm8z6ay3vj4d64ashxkgxcyhjewwsvgvelj": 22449169,
      "ed25519_pk1qq9l2qrqazk5fp4kt2kvjtsjc32g0ud888um8k2pvms0cw2r0uzsute83u": 1787992,
      "ed25519_pk1qqx6h559ee7pa67dm255d0meekt6dmq6857x302wdwrhzv47z9hqucdnt2": 369024,
    }
  }
]
```

## Get rewards history for some epochs

Get the rewards history of the *length* last epoch(s) from tip.

```sh
jcli rest v0 rewards history get <length> <options>
```

- \<length\> - number of epochs, starting from the last epoch from tip, to get the reward history for.

The options are

- -h <node_addr> - see [conventions](#conventions)
- --debug - see [conventions](#conventions)

```sh
jcli rest v0 rewards history get 2 -h <node_addr>
```

```json
[
  {
    "epoch": 93,
    "drawn": 3835616440000,
    "fees": 641300000,
    "treasury": 467151470296,
    "stake_pools": {
      "0087011b9c626759f19d9d0315a9b42492ba497438c12efc026d664c9f324ecb": [
        1121750881,
        21771124247
      ],
      "014bb0d84f40900f6dd85835395bc38da3ab81435d1e6ee27d419d6eeaf7d16a": [
        429241408,
        8155586765
      ],
      "01bd272cede02d0b0c9cd47b16e5356ab3fb2330dd9d1e972ab5494365309d2a": [
        1691506850,
        32829041110
      ],
    },
    "accounts": {
      "ed25519_pk1002kje4l8j7kvsseyauusk3s7nzef4wcvvafltjmg0rkzr6qccyqg064kz": 33311805,
      "ed25519_pk100549kxqn8tnzfzr5ndu0wx7pp2y2ck28mnykq03m2z5qcwkvazqx9fp0h": 15809,
      "ed25519_pk10054y058qfn5wnazalnkax0mthg06ucq87nn9320rphtye5ca0xszjcelk": 10007789,
      "ed25519_pk10069dsunppwttl4qtsfnyhjnqwkunuwxjxlandl2fnpwpuznf5pqmg3twe": 545094806,
      "ed25519_pk1009sfpljfgx30z70l3n63gj7w9vp3epugmd3vn62fyr07ut9pfwqjp7f8h": 4208232,
    },
  },
  {
    "epoch": 92,
    "drawn": 3835616440000,
    "fees": 620400000,
    "treasury": 480849578351,
    "stake_pools": {
      "0087011b9c626759f19d9d0315a9b42492ba497438c12efc026d664c9f324ecb": [
        979164601,
        19003786459
      ],
      "0105449dd66524111349ef677d1ebc25247a5ba2d094913f52aa4db265eac03a": [
        26977274,
        972170279
      ],
      "014bb0d84f40900f6dd85835395bc38da3ab81435d1e6ee27d419d6eeaf7d16a": [
        299744265,
        5695141053
      ],
    },
    "accounts": {
      "ed25519_pk1002kje4l8j7kvsseyauusk3s7nzef4wcvvafltjmg0rkzr6qccyqg064kz": 40581616,
      "ed25519_pk100549kxqn8tnzfzr5ndu0wx7pp2y2ck28mnykq03m2z5qcwkvazqx9fp0h": 49156,
      "ed25519_pk10054y058qfn5wnazalnkax0mthg06ucq87nn9320rphtye5ca0xszjcelk": 12306084,
      "ed25519_pk10069dsunppwttl4qtsfnyhjnqwkunuwxjxlandl2fnpwpuznf5pqmg3twe": 142737175,
      "ed25519_pk1009sfpljfgx30z70l3n63gj7w9vp3epugmd3vn62fyr07ut9pfwqjp7f8h": 3932910,
    },
  }
]
```

---

## Get voting committee members

Get the list of voting committee members.

```sh
jcli rest v0 vote active committees get <options>
```

The options are

- -h <node_addr> - see [conventions](#conventions)
- --debug - see [conventions](#conventions)
- --output-format \<format\> - see [conventions](#conventions)

YAML printed on success

```yaml
---
- 7ef044ba437057d6d944ace679b7f811335639a689064cd969dffc8b55a7cc19 # list of members
- f5285eeead8b5885a1420800de14b0d1960db1a990a6c2f7b517125bedc000db
```

## Get active voting plans and proposals

Get the list of active voting plans and proposals.

```sh
jcli rest v0 vote active plans get <options>
```

The options are

- -h <node_addr> - see [conventions](#conventions)
- --debug - see [conventions](#conventions)
- --output-format \<format\> - see [conventions](#conventions)

YAML printed on success

```yaml
---
- committee_end:
    epoch: 10
    slot_id: 0
  proposals:
    - external_id: adb92757155d09e7f92c9f100866a92dddd35abd2a789a44ae19ab9a1dbc3280
      options:
        OneOf:
          max_value: 3
    - external_id: 6778d37161c3962fe62c9fa8a31a55bccf6ec2d1ea254a467d8cd994709fc404
      options:
        OneOf:
          max_value: 3
  vote_end:
    epoch: 5
    slot_id: 0
  vote_start:
    epoch: 1
    slot_id: 0
```
