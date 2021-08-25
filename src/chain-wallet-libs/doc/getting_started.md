# Jörmungandr Wallet and Chain interactions

- list of jörmungandrwallet lib functions
- what and how are used for/to interact with the external network.

## Wallet used in examples

```json
"wallet": {
  "Note":      "DAEDALUS",
  "Mnemonics": "tired owner misery large dream glad upset welcome shuffle eagle pulp time",
  "Funds":
  [
    {
      "Address": "DdzFFzCqrhsktawSMCWJJy3Dpp9BCjYPVecgsMb5U2G7d1ErUUmwSZvfSY3Yjn5njNadfwvebpVNS5cD4acEKSQih2sR76wx2kF4oLXT",
      "Value":   10000,
    },
    {
      "Address": "DdzFFzCqrhsg7eQHEfFE7cH7bKzyyUEKSoSiTmQtxAGnAeCW3pC2LXyxaT8T5sWH4zUjfjffik6p9VdXvRfwJgipU3tgzXhKkMDLt1hR",
      "Value":   10100,
    },
    {
      "Address": "DdzFFzCqrhsw7G6njwb8FTBxVCh9GtB7RFvvz7KPNkHxeHtDwAPT2Y6QLDLxVCu7NNUQmwpAfgG5ZeGQkoWjrkbHPUeU9wzG3YFpohse",
      "Value":   1, // this will be ignored upon conversion
    },
  ]
}
```

### The fees in the sample chain

```json
  "fees": {
    "certificate": 5,
    "coefficient": 3,
    "constant": 2
  },
```

---

## Main core api

- [wallet_recover](#wallet_recover)
- [wallet_id](#wallet_id)
- [wallet_retrieve_funds](#wallet_retrieve_funds)
- [wallet_total_value](#wallet_total_value)
- [wallet_convert](#wallet_convert)
- [wallet_convert_ignored](#wallet_convert_ignored)
- [wallet_convert_transactions_size](#wallet_convert_transactions_size)
- [wallet_convert_transactions_get](#wallet_convert_transactions_get)
- [wallet_set_state](#wallet_set_state)
- [wallet_vote_proposal](#wallet_vote_proposal)
- [wallet_vote_cast](#wallet_vote_cast)

---

### wallet_recover

- **input**:
  - `mnemonics` - utf8 string (already normalized **NFKD**) in **english**
  - `password` - optional
- **output**:
  - `wallet` - recovered from mnemonics
  - `error` - returned in case the mnemonics are not valid (invalid length or checksum)

> **NFKD** - **N**ormalization **F**orm: Compatibility (**K**) **D**ecomposition.
> Characters are decomposed by compatibility, and multiple combining characters are arranged *in a specific order*.

Can also be used to recover a wallet even after you have transferred all the funds to the new format (see the [wallet_convert](#wallet_convert) function), just keep in mind that it will reset the wallet internal state.

---

### wallet_id

- **input**:
  - `wallet` - retireved from mnemonics ([wallet_recover](#wallet_recover))
- **output**:
  - `walled_id` - (aka accountID) used against the blockchain/explorer to retrieve the state of the wallet
  - `error` - returned in case the wallet is not yet recovered

This is the wallet identifier (aka accountID) that is to be used against the blockchain/explorer
to retrieve the state of the wallet (counter, total value, ...)

#### How to query the network for the wallet_id information

API calls used:
>Get: /v0/**account/{wallet_id}**
>
> - **wallet_id** value has to be a string in hexadecimal encoding

The network can be queried by using the Rest API interface.

> - Node rest api interface is at **http://127.0.0.1:8001/api**
> - wallet_id is **daa45bd19e1f6d2fce25542c2bda11076d4b359d3b5a6dd5b9588e2f34569503**

```json
Request {
    method: GET,
    url: "http://127.0.0.1:8001/api/v0/account/daa45bd19e1f6d2fce25542c2bda11076d4b359d3b5a6dd5b9588e2f34569503",
    headers: {},
}
```

When issued the first time, the response body of that query will be empty and you will get status **404 Not Found**
since the account doesn't exist yet on the chain:

```json
Response {
    url: "http://127.0.0.1:8001/api/v0/account/daa45bd19e1f6d2fce25542c2bda11076d4b359d3b5a6dd5b9588e2f34569503",
    status: 404,
    headers: {
        "content-length": "0",
        "date": "Thu, 14 May 2020 18:51:12 GMT",
    },
}
```

When issued after the convert process is completed:

```json
Response {
    url: "http://127.0.0.1:8001/api/v0/account/daa45bd19e1f6d2fce25542c2bda11076d4b359d3b5a6dd5b9588e2f34569503",
    status: 200,
    headers: {
        "content-type": "application/json",
        "content-length": "91",
        "date": "Thu, 14 May 2020 19:24:54 GMT",
    },
}
Response body:
{
  "counter": 0,
  "delegation": {
    "pools": []
  },
  "last_rewards": {
    "epoch": 0,
    "reward": 0
  },
  "value": 20089
}
```

The interesting parts here are the **counter** and **value** that will be used from [wallet_set_state](#wallet_set_state)
and from other interactions with the node (vote casting, ...)

---

### wallet_retrieve_funds

- **input**:
  - `wallet` - recovered from mnemonics ([wallet_recover](#wallet_recover))
  - `block0` - content that has to be retrieved from the blockchain network using the rest api
- **output**:
  - `wallet` - with initial funds retrieved from block0 (not yet converted)
  - `settings` - contains some ledger parameters retrieved from block0 (fees, block0 hash, block0 date, ...)
  - `error` - returned in case of failure (wallet not yet recovered, block0 corrupted, ...)

This function is used to retrieve the funds from *yoroi* wallet that are available in the given *block0*.

> This function may take some time to complete.

The required *block0* content has to be retrieved from the network by querying the Rest API interface.

#### How to retrieve block0 content from the network

API calls used:
>Get: /v0/**settings**
>
>Get: /v0/**block**/{**block_hash**}

In order to retrieve the block0 content from the chain, we need to get the block0 hash first.
This can be archieved by querying the **settings** endpoint.

```json
Request {
    method: GET,
    url: "http://127.0.0.1:8001/api/v0/settings",
    headers: {},
}

Response {
    url: "http://127.0.0.1:8001/api/v0/settings",
    status: 200,
    headers: {
        "content-type": "application/json",
        "content-length": "599",
        "date": "Thu, 14 May 2020 19:52:04 GMT",
    },
}
Response body:
{
  "block0Hash": "6e9557ffc31841dc53bc421cd137ee02ccdd26d6857e93a5c00fbcaced87f338",
  "block0Time": "2020-05-01T00:00:00+00:00",
  "blockContentMaxSize": 102400,
  "consensusVersion": "bft",
  "currSlotStartTime": null,
  "epochStabilityDepth": 102400,
  "fees": {
    "certificate": 5,
    "coefficient": 3,
    "constant": 2
  },
  "rewardParams": {
    "compoundingRatio": {
      "denominator": 1,
      "numerator": 0
    },
    "compoundingType": "Linear",
    "epochRate": 4294967295,
    "epochStart": 0,
    "initialValue": 0,
    "poolParticipationCapping": null,
    "rewardDrawingLimitMax": "None"
  },
  "slotDuration": 10,
  "slotsPerEpoch": 6,
  "treasuryTax": {
    "fixed": 0,
    "ratio": {
      "denominator": 1,
      "numerator": 0
    }
  }
}
```

What we need is **block0Hash** value, **6e9557ffc31841dc53bc421cd137ee02ccdd26d6857e93a5c00fbcaced87f338** in the example,
that can be used to get the *block0* content by querying the **block** endpoint.

```json
Request {
    method: GET,
    url: "http://127.0.0.1:8001/api/v0/block/6e9557ffc31841dc53bc421cd137ee02ccdd26d6857e93a5c00fbcaced87f338",
    headers: {},
}

Response {
    url: "http://127.0.0.1:8001/api/v0/block/6e9557ffc31841dc53bc421cd137ee02ccdd26d6857e93a5c00fbcaced87f338",
    status: 200,
    headers: {
        "content-type": "application/octet-stream",
        "content-length": "1257",
        "date": "Thu, 14 May 2020 20:09:16 GMT",
    },
}
Response body:
00520000000004950000000000000000000000002ba03ed6c06c6dfa2f3f575978352a9c5f38ac5c27db8d5bf7c4f1c0bb67c93d000000000000000000000000000000000000000000000000000000000000000000af0000000f0088000000005eab668000410100c20001039800000000000000020000000000000003000000000000000501040000000601410a04040000a8c0020800000000000000640244000190000184000190000581000448000000000000000004a0000000000000000000000000000000000000000000000001000000000000000004c8000000000000000002e0e57ceb3b2832f07e2ef051e772b62a837f7a486c35e38f51bf556bd3abcd8eca00590001010000000000002710004c82d818584283581c3657ed91ad2f25ad3ebc4faec404779f8dafafc03fa181743c76aa61a101581e581cd7c99cfa13e81ca55d026fe0395124646e39b188c475fb276525975d001ab75977f200590001010000000000002774004c82d818584283581c11d8b9351de0ba3f9de05fa93eea0ff7f47a3b660ebd80e33f1b35eea101581e581cd7c99cfa13e81caec26acce0efc8a5113b91748bd0391ebf9c69813a001a65a9ad5000590001010000000000000001004c82d818584283581c847329097386f263121520fc9c364047b436298b37c9148a15efddb4a101581e581cd7c99cfa13e81ce17f4221e0aed54c08625a0a8c687d9748f462a6b2001af866b8b900590001010000000000004e20004c82d818584283581c0992e6e3970dd01055ba919cff5b670a6813f41c588eb701231e3cf0a101581e581c4bff51e6e1bcf245c7bcb610415fad427c2d8b87faca8452215970f6001a660a147700590001010000000000004e84004c82d818584283581c4baebf60011d051b02143a3417514fed6f25c8c03d2253025aa2ed5fa101581e581c4bff51e6e1bcf245c7bcb5104c7ca9ed201e1b1a6c6dfbe93eadeece001a3189727000590001010000000000004ee8004c82d818584283581cffd85f20cf3f289fd091e0b033285ecad725496bc57035a504b84a10a101581e581c4bff51e6e1bcf245c7bcb4105299a598c50eabacdd0f72815c016da7001a57f9068f004f0001010000000000004f4c004282d818583883581c6385953676247219ea840d7e5640ad6ba3c7606b77501281379642a7a10155544be5d019220eaee07eba7564bed7564a33654315001a8eabfbba004f0001010000000000004fb0004282d818583883581c3e1618d2f7d3d70f89f6d3fd9ba764f87ff351b3b4e8d9197c8c4909a10155544be5d319cf264a1a26befada0d92c67c0e8a83b5001a12ecc1a3004f0001010000000000000001004282d818583883581c69029f227e622c8963a6b0b217fa5b25337303ac9c64e897795cae32a10155544be5d21940c9c0b1431279b348a9f66bc57dc32a001a9c8a563f00380001010000000000007530002b82d818582183581c783fd3008d0d8fb4532885481360cb6e97dc7801c8843f300ed69a56a0001a7d83a21d00380001010000000000000001002b82d818582183581cadff678b11b127aef0c296e88bfb4769c905284716c23e5d63278787a0001a63f679c700590001010000000000009c40004c82d818584283581cfd50a82edd71fef2210d2add4b4b476a44d86f84a866a1d571d31302a101581e581c06b477a547702178ca90ef8a597adf4e777fb189774bf177234ca772001a059f1bbf
```

The response content is the block0 **binary** content that can be used as input to retrieve the funds.
> Depending on the binding libs the content encoding may need to be changed before hand.

---

### wallet_total_value

- **input**:
  - `wallet` - with initial funds retrieved ([wallet_retrieve_funds](#wallet_retrieve_funds))
- **output**:
  - `total_value` - of retrieved initial funds (it includes values that may be ignored later upon conversion)
  - `error` - internal

This function is used get the total value of the funds (UTxO's) in the wallet.
Make sure to call [wallet_retrieve_funds](#wallet_retrieve_funds) prior to calling this function
otherwise you will always have **0** as a returned value.

- In the example wallet **total_value** = **20101**

> Keep in mind that this may not be the total value that will be converted, since some UTxO's may be ignored on convert.

---

### wallet_convert

- **input**:
  - `wallet` - with initial funds retrieved from block0 ([wallet_retrieve_funds](#wallet_retrieve_funds))
  - `settings` - ledger parameters retrieved from block0 ([wallet_retrieve_funds](#wallet_retrieve_funds))
- **output**:
  - `conversion` - contains a list of *ignored UTxO*'s and a list of *transactions* built to convert the retrieved wallet
  - `error` - returned in case of failure (wallet not yet recovered, funds not yet retrieved, ...)

This function is used to convert the existing funds to the new wallet
once the funds have been retrieved with [wallet_retrieve_funds](#wallet_retrieve_funds).

---

### wallet_convert_ignored

- **input**:
  - `conversion` - contains *ignored UTxO*'s list built from conversion ([wallet_convert](#wallet_convert))
- **output**:
  - `ignored_value` - the total value of the *ignored UTxO*'s lost into dust inputs
  - `ignored_utxo_cnt` - the number of *ignored UTxO*'s (dust UTxO)
  - `error` - returned in case of failure (wallet not yet converted, ...)

This function is used to get the total value ignored in the conversion.
These returned values are informational only and this show that there are UTxOs entries
that are unusable because of the way they are populated with dust.

> The cost of converting those UTxOs is higher than the value itself.

- In the example wallet **ignored_value** = **1** and **ignored_utxo_cnt** = **1**

---
> Keep in mind that the value that will be converted is:
>
> - ([wallet_total_value](#wallet_total_value)) **total_value** - **ignored_value** (ex: 20101 - 1 = **20100**)
>

---

### wallet_convert_transactions_size

- **input**:
  - `conversion` - contains *transactions* list built from conversion ([wallet_convert](#wallet_convert))
- **output**:
  - `size` - the number of *transactions* built to convert the retrieved wallet

This function is used to get the number of transactions built
to convert the retrieved wallet by [wallet_convert](#wallet_convert).

> There is a limit on the number of inputs a transaction can carry, hence there may be more than one.

---

### wallet_convert_transactions_get

- **input**:
  - `conversion` - contains *transactions* list built from conversion ([wallet_convert](#wallet_convert))
  - `index` - index-nth transaction in the conversion, from 0 to `size-1` where size is retrieved from ([wallet_convert_transactions_size](#wallet_convert_transactions_size))
- **output**:
  - `transaction` - single transaction data in bytes that will be sent to the network
  - `error` - returned in case of failure (wallet not yet converted, ...)

This function is used to get the built transactions one by one.
The retrieved transactions can be sent to the network by using the Rest API.

#### How to send a transaction to the network

The retrieved transactions can be send to the network one by one.

API calls used:
>Post: /v0/**message**

The request body is the **binary** content of a transaction.

```json
Request {
    method: POST,
    url: "http://127.0.0.1:8001/api/v0/message",
    headers: {
        "content-type": "application/octet-stream",
    },
}
Request body:
0181000202010000000000000027102172f417efc6f6cf700874f7d34cd1d88fcdde8a7b520a4a8dab4cd19f72f75e0000000000000027747b1bc906421ea52dcd3caeda61987061dfaca8b82fe8129574cbc90090dfe18905daa45bd19e1f6d2fce25542c2bda11076d4b359d3b5a6dd5b9588e2f345695030000000000004e79009c6763b7a25e9621bd9147477a550c2db25bf06f910da33d99bd8023489b5844f4dc2ca7c1c2e4d32b640ddd26fbdaebbc165f16f988adfef0dce53c658d2522d0d9dc0d7560185a4cc0430fd175a7c5b60311078290fe013ac4ce110651feff551c15e45b0ee247ed9a1fd3d8ea1fa16307deb26325db796cebef5140a4d60300563c63eca20c7afc3aa068896bed32cb8303605dcc7375259251f3237360ce3368ce7df024944796d76278723f017eb954cb8e9cab88f8e73d077f2378c662b05ff60a5745aeb462c3200ede615add80fadb621d6fb0d31fb5eae078e5e70ca6ecb0794c1fbacde773790902709a0f6759d838779cde3287fe24da0a561b1302

Response {
    url: "http://127.0.0.1:8001/api/v0/message",
    status: 200,
    headers: {
        "content-type": "text/plain; charset=utf-8",
        "content-length": "64",
        "date": "Thu, 14 May 2020 21:18:22 GMT",
    },
}
Response body:
68dcc12fe0dfe5e7b66ca6f8c959f9aa43b273e120a77fc3e4e2f04f1ecd7968
```

The response, **68dcc12fe0dfe5e7b66ca6f8c959f9aa43b273e120a77fc3e4e2f04f1ecd7968** in the example, contains
the **transactionID** that can be used further to check the status of the transaction if so desired.

#### How to check the transaction status on the network

API calls used:
>Get: /v0/**fragment**/**logs**

```json
Request {
    method: GET,
    url: "http://127.0.0.1:8001/api/v0/fragment/logs",
    headers: {},
}
Response {
    url: "http://127.0.0.1:8001/api/v0/fragment/logs",
    status: 200,
    headers: {
        "content-type": "application/json",
        "content-length": "331",
        "date": "Thu, 14 May 2020 21:33:19 GMT",
    },
}
Response body:
[
  {
    "fragment_id": "68dcc12fe0dfe5e7b66ca6f8c959f9aa43b273e120a77fc3e4e2f04f1ecd7968",
    "last_updated_at": "2020-05-14T19:24:50.036177914+00:00",
    "received_at": "2020-05-14T19:24:46.058286565+00:00",
    "received_from": "Rest",
    "status": {
      "InABlock": {
        "block": "e6de925d103d419ebf7cd8625699d9698cf3deeb34c5ae8d7ee828270ab42b50",
        "date": "19884.5"
      }
    }
  }
]
```

The response contains an array of transactions data and has to be filtered by **fragment_id**

---

### wallet_set_state

- **input**:
  - `wallet` - the converted wallet
  - `value` - the value of the converted wallet that should be retrieved from the network
  - `counter` - the wallet account counter that should be retrieved from the network and will be used for each transaction (vote casting, ...)
- **output**:
  - `wallet` - the updated wallet
  - `error` - returned in case of failure (wallet not yet converted, ...)

This function is used update the wallet account state.
This is the value retrieved from any node Rest API endpoint that allows to query
for the account state. It gives the **value** associated to the account as well as the **counter**.

It is important to be sure to have an updated wallet state before doing any
transactions otherwise future transactions may fail to be accepted by any
nodes of the blockchain because of invalid signature state.

#### How to query the network for wallet state

API calls used:
>Get: /v0/**account/{wallet_id}**
>
> - **wallet_id** value has to be a string in hexadecimal encoding

```json
Request {
    method: GET,
    url: "http://127.0.0.1:8001/api/v0/account/daa45bd19e1f6d2fce25542c2bda11076d4b359d3b5a6dd5b9588e2f34569503",
    headers: {},
}

Response {
    url: "http://127.0.0.1:8001/api/v0/account/daa45bd19e1f6d2fce25542c2bda11076d4b359d3b5a6dd5b9588e2f34569503",
    status: 200,
    headers: {
        "content-type": "application/json",
        "content-length": "91",
        "date": "Thu, 14 May 2020 19:24:54 GMT",
    },
}
Response body:
{
  "counter": 0,
  "delegation": {
    "pools": []
  },
  "last_rewards": {
    "epoch": 0,
    "reward": 0
  },
  "value": 20089
}
```

The **counter** and **value** are the field of interest that will be used to update the wallet state.

> **Note**: the initial value on the chain is different from the value that was to be converted
> because of the chain fees that were applied when the transactions are created and send to the chain.

---

### wallet_vote_proposal

- **input**:
  - `vote_plan_id` - the ID of the voteplan on the chain
  - `payload_type` - the payload of the voteplan (right now is just **Public** and 1 is the numeric value represented on chain)
  - `index` - the proposal index within the provided vote plan
  - `num_choices` - the number of choices allowed for this proposal (min:1 max: 16)
- **output**:
  - `proposal` - the builded proposal that we will cast the vote for
  - `error` - returned in case of failure (wrong vote_plan_id format, num_choices outside the allowed range, ...)

This function is used to build the proposal that we will cast the vote for with [wallet_vote_cast](#wallet_vote_cast).

#### How to retieve the voteplans info and data from the network

API calls used:
>Get: /v0/**vote/active/plans**

```json
Request {
    method: GET,
    url: "http://127.0.0.1:8001/api/v0/vote/active/plans",
    headers: {},
}
Response {
    url: "http://127.0.0.1:8001/api/v0/vote/active/plans",
    status: 200,
    headers: {
        "content-type": "application/json",
        "content-length": "3129",
        "date": "Mon, 01 Jun 2020 11:13:16 GMT",
    },
}
Response body:
[
  {
    "voteplan_id": "a19b9eae101a423dd3936c978ad44a145407be363f457ddb76d4cd6eb8847376",
    "vote_start": {
      "epoch": 0,
      "slot_id": 0
    },
    "vote_end": {
      "epoch": 5,
      "slot_id": 0
    },
    "committee_end": {
      "epoch": 10,
      "slot_id": 0
    },
    "payload_type": "Public",
    "proposals": [
      {
        "external_id": "5db05d3c7bfc37f2059d24966aa6ef05cfa25b6a478dedb3b93f5dca5c57c24a",
        "index": 0,
        "options": {
          "range": {
            "start": 0,
            "end": 3
          }
        }
      },
      {
        "external_id": "f78a5e1b0cc558529be705d58479602ce8fe7af1b11e8d383e0b112d2d58d3fe",
        "index": 1,
        "options": {
          "range": {
            "start": 0,
            "end": 3
          }
        }
      }
    ]
  },
  ...
  {
    "voteplan_id": "72b8904083ecf511f36940c03bf8b592c814d7aaa41d1ec8a67f9254ebd1f202",
    "vote_start": {
      "epoch": 0,
      "slot_id": 0
    },
    "vote_end": {
      "epoch": 5,
      "slot_id": 0
    },
    "committee_end": {
      "epoch": 10,
      "slot_id": 0
    },
    "payload_type": "Public",
    "proposals": [
      {
        "external_id": "df2596ad616577c9047d23f106371258c98c329a662432c1e57d80092ae74e44",
        "index": 0,
        "options": {
          "range": {
            "start": 0,
            "end": 3
          }
        }
      },
      {
        "external_id": "d9d3544bad57f2597f55fce907adfe4ad5afe1aedc3d5480a3745a200b153708",
        "index": 1,
        "options": {
          "range": {
            "start": 0,
            "end": 3
          }
        }
      }
    ]
  }
]
```

The interesting values are:

- VotePlan related:
  - **voteplan_id**
  - **payload_type**
- Proposal related:
  - **index**
  - options.range.**end** (to be used as `num_choices`)

---

### wallet_vote_cast

- **input**:
  - `wallet` - the converted wallet
  - `settings` - ledger parameters retrieved from block0 ([wallet_retrieve_funds](#wallet_retrieve_funds))
  - `proposal` - the proposal retrieved from [wallet_vote_proposal](#wallet_vote_proposal)
  - `choice` - the choice our vote, a numeric value [0 - [wallet_vote_proposal](#wallet_vote_proposal) num_choices)
- **output**:
  - `transaction` - single transaction data in bytes, containing the vote cast, that will be sent to the network
  - `error` - returned in case of failure (wrong choice value, ...)

This function is used to cast the vote choice for the proposal and to retrieve the already built transaction for that vote.
The retrieved transactions can be sent to the network by using the Rest API. [How to send a transaction to the network](#How-to-send-a-transaction-to-the-network)

> **Note**: Please make sure to have an updated wallet state, using [wallet_set_state](#wallet_set_state)
> before using this function.

---

## Cleanup core api

- [wallet_delete_wallet](#wallet_delete_wallet)
- [wallet_delete_settings](#wallet_delete_settings)
- [wallet_delete_conversion](#wallet_delete_conversion)
- [wallet_delete_proposal](#wallet_delete_proposal)

---

### wallet_delete_wallet

- **input**:
  - `wallet`
- **output**:
  - none

---

### wallet_delete_settings

- **input**:
  - `settings`
- **output**:
  - none

---

### wallet_delete_conversion

- **input**:
  - `conversion`
- **output**:
  - none

---

### wallet_delete_proposal

- **input**:
  - `proposal`
- **output**:
  - none

---

## Other/specific api

- [wallet_error_to_string](#wallet_error_to_string)
- [wallet_error_details](#wallet_error_details)
- [wallet_delete_error](#wallet_delete_error)
- [wallet_delete_string](#wallet_delete_string)
- [wallet_delete_buffer](#wallet_delete_buffer)

---

### wallet_error_to_string

- **input**:
  - `error`
- **output**:
  - error_string

---

### wallet_error_details

- **input**:
  - `error`
- **output**:
  - error_details

---

### wallet_delete_error

- **input**:
  - `error`
- **output**:
  - none

---

### wallet_delete_string

- **input**:
  - `*`
- **output**:
  - none

---

### wallet_delete_buffer

- **input**:
  - `*`
  - `length`
- **output**:
  - none

---
