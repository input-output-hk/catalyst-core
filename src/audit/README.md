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

### Retrieve historical fund results from node.

Takes several minutes to replay state and stabilize before it is possible to retrieve vote results ⌛

```bash
curl http://127.0.0.1:10000/api/v0/vote/active/plans > activevoteplans.json 
```

### Offline fragment analysis and tally

This tool allows facilitates *offline* fragment analysis of a fund using historical blockchain state.       

*Example usage:*

```
cargo build --release -p audit
```  

#### Cross reference offline regenerated tally with official published results.

```bash

OFFICIAL_RESULTS=/tmp/activevoteplans.json 
BLOCK0=/tmp/fund9-leader-1/artifacts/block0.bin
FRAGMENTS_STORAGE=/tmp/fund9-leader-1/persist/leader-1

./target/release/offline --fragments $FRAGMENTS_STORAGE --block0 $BLOCK0 --official-results $OFFICIAL_RESULTS

```

[See here for more details](src/offline/bin/README.md)

