### Reproduce published results
### Start and load live node with historical state

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

Takes several minutes to replay state and stabilize before it is possible to retrieve vote results ⌛

Try after ~5 mins, if it has not replayed and stabilized. 
The following error will be shown - `Internal server error: Blockchain tip not set in REST/RPC context`.

⏳

```bash
curl http://127.0.0.1:10000/api/v0/vote/active/plans > activevoteplans.json 
```

**activevoteplans.json** = FINAL RESULTS.

##### Make sure the jormungandr container has been stopped once you have successfully retrieved the results. 
```bash
sudo docker docker stop $JORMUNGANDR_CONTAINER_ID
```