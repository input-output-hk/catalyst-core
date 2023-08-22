# `leader0` Schedule

## `node_fetch_event`
    1. [ ] Look for event.yml in local storage
    2. [X] Fetch event with `start_time` in near future, otherwise, look for event row in event.yml
    3. [ ] If NODE_IS_RELOADABLE and event.yml differs from EventDB, reset, otherwise, emit warning.
    4. [ ] Run this task recurrently in the background.
## `node_wait_for_start_time`
    1. [X] Wait for `event.start_time` to pass.
## `node_fetch_host_keys`
    1. [X] Fetch keys for hostname and event. Create if missing.
## `node_fetch_leaders`
    1. [X] Fetch hostname information for leaders registered for this event.
## `node_set_secret`
    1. [X] Fetch the node secret key for the hostname for this event. Create if missing.
## `node_set_topology_key`
    1. [X] Fetch the node P2P topology key for the hostname for this event. Create if missing.
## `node_set_config`
    1. [X] Fetch the node configuration for the hostname for this event. Create if missing.
## `event_snapshot_period`
    1. [X] Run IdeaScale snapshot importer recurrently until snapshot `is_final`.
    2. [X] Run DBSync snapshot importer recurrently until snapshot `is_final`.
## `node_snapshot_data`
    1. [-] Collect registered voters
    2. [-] Collect individual contributions
    3. [-] Collect voting groups
    4. [-] Collect proposals
## `block0_tally_committee`
    1. [X] Fetch tally committee information. Create if missing.
    1.1 [X] Fetch tally committee keys. Create if missing.
    1.2 [-] Fetch tally committee member keys. Create if missing.
## `block0_voting_tokens`
    1. [ ] Build a voting token for each voting group for this event: `list[(Token, VotingGroup)]`.
## `block0_wallet_registrations`
    1. [ ] Convert voter.voting_key into Bech32 (check if raw bytes are ed25519_pk or ed25519_sk, use jcli to create single address).
    2. [ ] Create `list[InitialFund]` with from voters.
## `block0_token_distributions`
    1. [ ] For each item in `list[Voter]`, use `voter.voting_group` to fetch the token from `list[(Token, VotingGroup)]`.
    2. [ ] Calculate the token value from the 
## `block0_vote_plans`
    1. [ ] Build a voting token for each voting group.
## `block0_genesis_build`
    1. [-] Collect initial Fund fragments for committee.
    2. [-] Collect initial Fund fragments for registered wallets.
    3. [-] Collect initial Token fragments for contributions.
    4. [ ] Collect initial SignedCertificate fragments from voteplans.
    5. [X] Generate `genesis.yaml` file, convert to `block0.bin`.
## `block0_publish`
    1. [X] Insert `block0.bin` raw bytes into `event.block0` in EventDB.
## `node_wait_for_voting`
    1. [X] Wait for `event.voting_start` to pass.
## `event_voting_period`
    1. [X] Start `jormungandr` node as a concurrent subprocess.
## `wait_for_tally`
    1. [ ] Wait for `event.voting_end` to pass.
## `event_tally_period`
    1. [ ] Execute tally for proposals.
## `node_cleanup`
    1. [ ] Archive event data, and reset local storage for future events.
