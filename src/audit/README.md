# Audit Tooling:

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

earthly +all
docker run  --net=host -v $MOUNT_PATH --env STORAGE_PATH=$HISTORICAL_STATE --env GENESIS_PATH=$BLOCK_0 jormungandr
```

Curl for active vote plan to recreate IOG results.

```bash
curl http://127.0.0.1:10000/api/v0/vote/active/plans > vote_plans_replayed.txt
```
