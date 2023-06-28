# Audit Tooling:

## Online audit

During the voting event, it is possible to keep track of the *tally* and *votes cast* for each proposal.


```bash

curl https://core.projectcatalyst.io/api/v0/vote/active/plans

```

```yaml

- index: 45
      options:
        end: 2
        start: 0
      proposal_id: aa5bd491d8c67b064bfda10c52d2c5d942e9e993048425c9086a1b7cf4aa0c89ad8
      tally:
        Private:
          state:
            Encrypted:
              encrypted_tally: aGl0sY+JrffAz2+C8LlTjJY6BLvfcCqtiGNnYRsiy2q0I4oJAAAAAKR7hDvDnv0nIeplwiYoWFwWWKL+xp7Cv/Wq7p3mcH0w0DvAmYIRrHbKMiJXfq97z+e5uf3JTiAY2gvtsztrK4xyke4Q7w579JyHqZcImKFhcFlii/saewr/1qu6d5nB9MH5pR4vp/kn4dJo9MAW65NiRiL9vRkj3cUblUvWbyXI4
      votes_cast: 1053

```

During the voting event, the endpoint will return an encrypted tally. We will publish the *decrypt shares* and *public keys*, allowing you to decrypt and verify the process results. [See live tooling for more info](src/live/README.md)

Once the voting event terminates this endpoint will return the *final* decrypted results which you can then verify with catalyst [offline tooling](src/offline/bin/README.md).

```yaml
---
index: 251
proposal_id: 5b70475324adcf73bd2da4247b2e7754464da8178c7206a1a17be50f0e68f404b1
options:
  start: 0
  end: 2
tally:
  Private:
    state:
      Decrypted:
        result:
          results:
          - 64822000
          - 25450000
          options:
            start: 0
            end: 2
votes_cast: 305
```

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

### Retrieve historical fund results *active vote plans* 

Takes several minutes to replay state and stabilize before it is possible to retrieve vote results ⌛

```bash
curl http://127.0.0.1:10000/api/v0/vote/active/plans > vote_plans_replayed.json
```

### Extract and analyse tally fragments and match with official *results* 
[See here](src/offline/bin/README.md)

