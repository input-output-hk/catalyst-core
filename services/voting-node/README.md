# Voting Node

This document describes the configuration of the voting node.

> 🚧 Note this document is a work in progress and will be updated as we go along.

- [Voting Node](#voting-node)
  - [Configuration](#configuration)
    - [Environment variables required by all nodes](#environment-variables-required-by-all-nodes)
        - [Optional](#optional)
    - [Environment variables required by `LEADER0` nodes](#environment-variables-required-by-leader0-nodes)
      - [Secret Generation](#secret-generation)
      - [IdeaScale Data Importer](#ideascale-data-importer)
      - [Dbsync Snapshot Data Importer](#dbsync-snapshot-data-importer)
      - [Optional](#optional-1)
  - [Voting Event](#voting-event)
    - [Stages](#stages)

## Configuration

### Environment variables required by all nodes

Use the following environment variables to configure the voting node:

* `EVENTDB_URL` - The URL of the event database

##### Optional

* `IS_NODE_RELOADABLE` - If set, the voting node will reload its configuration (optional). Defaults to `true`
* `VOTING_HOST` - Voting node IP (optional). Defaults to `0.0.0.0`
* `VOTING_PORT` - Voting node port (optional). Defaults to `8000`
* `VOTING_LOG_LEVEL` - Log level (optional). Defaults to `info`
* `VOTING_LOG_FORMAT` - Log format (optional). Defaults to `text`
* `VOTING_NODE_STORAGE` - Path to node storage (optional). Defaults to `./node_storage`
* `JORM_PATH` - Path to jormungandr executable (optional). Defaults to `jormungandr`
* `JCLI_PATH` - Path to jcli executable (optional). Defaults to `jcli`

### Environment variables required by `LEADER0` nodes

#### Secret Generation

* `COMMITTEE_CRS` - The CRS is used to generate committee members, this is only used by leader0
* `SECRET_SECRET` - The password used to encrypt/decrypt secrets in the database

#### IdeaScale Data Importer

* `IDEASCALE_API_TOKEN` - API token for IDEASCALE
* `IDEASCALE_API_URL` - URL for IdeaScale. Example: `https://cardano.ideascale.com`.

#### Dbsync Snapshot Data Importer

* `GVC_API_URL` - URL for GVC.
* `SNAPSHOT_OUTPUT_DIR` - Path to directory where snapshot data will be stored.
* `SNAPSHOT_NETWORK_IDS` - Network IDs (separated by space) for snapshot data. Possible values are `mainnet` and `testnet`.
* `DBSYNC_SSH_HOST_KEY` -
* `DBSYNC_SSH_PRIVKEY` -
* `DBSYNC_SSH_PUBKEY` -

#### Optional

* `SNAPSHOT_TOOL_PATH` - Path to snapshot tool executable (optional). Defaults to `snapshot_tool`
* `CATALYST_TOOLBOX_PATH` - Path to toolbox executable (optional). Defaults to `catalyst-toolbox`.
* `SNAPSHOT_INTERVAL_SECONDS` - Interval in seconds for snapshot data (optional).

## Voting Event

### Stages

* Preparation
* Fund Launch
* Proposal Submissions
* Proposal Community Review
* Registration
* Snapshot
* Voting
* Tallying
* Publishing
* Onboarding
* Rewarding
* Cooldown
* Next Fund Launch
