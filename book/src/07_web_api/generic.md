# Generic voting

## Pre-voting stage

- It is assumed that all vote plans on the blockchain are already present in the genesis block, Json-encoded vote plan(s) loaded from file. 
- Proposal submission. In order to submit a proposal for funding, a project owner submits a special proposal transaction to the blockchain.

## Voting stage

In order to cast a vote, a user must first obtain a *proposal_id* and *vote_plan_id*.

```kroki-seqdiag

seqdiag {
  client  -> node [label = "GET /api/v0/vote/active/plans"];
  client <-- node [label ="(vote_plan_id, proposal_id,*)"];
}
    
```

### Casting a vote

- The Client creates and signs a vote *fragment*, then sends it to the node.

```kroki-nomnoml

[Vote Fragment| choice;blockchain_metadata; vote_plan_id;proposal_id;signature]
    
```

```kroki-seqdiag

seqdiag {
  client  -> node [label = "POST /api/v0/message"];
  node  -> cluster [label = "propagate TX( Fragment )"
  ];
  node <-- cluster [rightnote="Block(TX) mined and stored"];
  client <-- node [label ="200"];
}
    
```

### Vote fragment in more detail

[Fragment breakdown](https://github.com/input-output-hk/catalyst-core/blob/main/src/chain-libs/chain-impl-mockchain/doc/format.md#type-11-vote-cast)

### Node verification
- The Node verifies if the fragment is correctly signed and formatted and then performs additional verification:
  - (*votes duplications, missing vote plan etc*)
- **/api/v1/fragments** is used for the same purpose but can also send batches of fragments

### Relevant endpoints

- get /v0/account/${id} **retrieve voting power**
- get /v1/votes/plan/account-votes/${account_id} **for history of votes**
- get /v0/settings **for block-hash and fees**
- get /v0/vote/active/plans **for votplan id to be able to construct vote** ✅
- get /v0/node/stats **for information about network congestion**
- post /v0/message **sending single fragment** ✅
- post /v1/fragments **sending batch fragments** ✅
- get /v1/fragments/statuses **checking fragment statuses**

## Posix style API

Generic API abstraction with multiple backends

## Vote plan[/vote_plan/{id}]

- Parameters
  - vote_plan_id: abc123 (required) - Unique identifier for a vote plan

## Get a vote plan [GET]

Gets a single vote plan by its unique identifier.

- Response 200 (application/json)
  - Attributes
  - vote_plan_id: abc123
  - title: vote plan
  - content: [proposals]

## Store vote[/store/{choice, vote_plan_id, proposal_id, signature}]

- Parameters
  - choice: 1
  - vote_plan_id: 2
  - proposal_id: 3
  - signature: 4

## Store vote [POST /vote]

Send vote fragment to Node

- Request (application/json)
  - choice
  - vote_plan_id
  - proposal_id
  - signature

- Response 201
  - Headers


## Post-voting stage

### Joint decryption of tally.