# Mechanics Of Voting

### Pre-requisites

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

[Fragment| choice;blockchain_metadata; vote_plan_id;proposal_id;signature]
    
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

- The Node verifies if the fragment is correctly signed and formatted and then performs additional verification (*votes duplications, missing vote plan etc*)

- **/api/v1/fragments** is used for the same purpose but can also send batches of fragments


### Relevant endpoints

- get  - /v0/account/${id} - retrieve voting power
- get  - /v1/votes/plan/account-votes/${account_id} - for history of votes
- get  -  /v0/settings - for block-hash and fees
- get  -  /v0/vote/active/plans - for votplan id to be able to construct vote ✅ 	
- get  -  /v0/node/stats - for information about network congestion
- post  - /v0/message - sending single fragment ✅ 	
- post  - /v1/fragments -  sending batch fragments ✅ 	
- get   - /v1/fragments/statuses - checking fragment statuses