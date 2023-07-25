# Audit Tooling:

## Offline audit

### Download Fund State
Download historical fund state from [here](https://github.com/input-output-hk/catalyst-core) in order to replay the voting event.

### Start and load node with historical state

Pre-requisites
- Install [Earthly CLI](https://earthly.dev/get-earthly)
- Docker or Podman
- Git

```bash
cd balance

# Mount local path as a volume in the container
MOUNT_PATH=/tmp/fund9-leader-1:/leader1stuff

HISTORICAL_STATE=/leader1stuff/persist/leader-1
BLOCK_0=/leader1stuff/artifacts/block0.bin

earthly +build && earthly +docker
docker run  --net=host -v $MOUNT_PATH --env STORAGE_PATH=$HISTORICAL_STATE --env GENESIS_PATH=$BLOCK_0 jormungandr
```

### Spin up node to retrieve vote results

```bash

Takes several minutes to replay state and stabilize before it is possible to retrieve vote results ⌛


curl http://127.0.0.1:10000/api/v0/vote/active/plans > activevoteplans.json 
```

### Offline fragment analysis and tally

This tool facilitates *offline* fragment analysis of a fund using historical blockchain state.     

##### Make sure the jormungandr container has been stopped in the context of node replay to re-generate the results.
```bash
sudo docker docker stop $JORMUNGANDR_CONTAINER_ID
```

*Example usage:*

```
cargo build --release -p audit
```  

*Cross reference offline tallies with **published** catalyst tallies.*

```bash

OFFICIAL_RESULTS=/tmp/activevoteplans.json 
BLOCK0=/tmp/fund9-leader-1/artifacts/block0.bin
FRAGMENTS_STORAGE=/tmp/fund9-leader-1/persist/leader-1

./target/release/offline --fragments $FRAGMENTS_STORAGE --block0 $BLOCK0 --official-results $OFFICIAL_RESULTS

```

This will create three files:
- *ledger_after_tally.json* **(decrypted ledger state after tally)** *should match official results!*
- *ledger_before_tally.json* **(encrypted ledger state before tally)** 
- *decryption_shares.json* **(decryption shares for each proposal)**

[See here for next steps of audit process](src/tally/README.md)

