# Voting Node Service

This service acts as a wrapper around a `jormungandr`, interacting with the Event DB to gather the necessary data so that the node can join in the network.


# Running locally

## Requirements

* git
* python 3.11
* poetry [https://python-poetry.org/docs/#installation](https://python-poetry.org/docs/#installation)
* jormungandr
* jcli
* Access to postgres with Event DB.

## 1. Get a working copy of the repository and change into the new directory.

```shell
git clone https://github.com/input-output-hk/catalyst-core.git
```

```shell
cd catalyst-core/services/voting-node
```

## 2. Initialize and enter virtual environment

To enable loading envvars from the `.env` file, the `poetry-dotenv-plugin` is recommended:

###
```shell
poetry self add poetry-dotenv-plugin
```

With this in place, poetry provides a shell that automatically loads the `.env` file, as well as activating the virtual environment in `.venv/bin/activate`

###
```shell
poetry shell
```

To exit the virtual environment, just type:

###
```shell
exit
```

### 3. Install `voting_node` package

Install the package defined in the `pyproject.toml` file.

```
poetry install
```

## 4. Run the voting-node

A quick way to run the latest jormungandr/jcli executables is to set their paths. If the paths are omitted, the default paths are `jormungandr` and `jcli`. The default log level is `info` by pythonic default, but for development mode, `debug` should be preferred.

### `voting-node` commands

To get a list of avaiable commands, just type:

```shell
voting-node --help
```
or simply,
```shell
voting-node
```
#### `voting-node start` command

This command starts the service API and executes the schedule according to the node's hostname.

If a node has the exact name `leader0`, it will run scheduled tasks with the purpose of creating `block0` for the current voting event. There is only one `leader0` node per event.

If a node has a name pattern `leader[0-9]+`, it will run scheduled tasks with the purpose minting blocks in the p2p network.

If a node has a name pattern `follower[0-9]+`, it will run scheduled tasks with the purpose following leader nodes.

To get a full listing of the arguments and options for this command, type:

```shell
voting-node start --help
```
```
Usage: voting-node start [OPTIONS]                                                                                                                                                                          

  Starts the voting node.

Options:
  --reloadable                    Enables the node to reload when it detects
                                  changes to the current Voting Event.    If
                                  not set, the node will still detect changes
                                  to the Voting Event, but will use    the
                                  configuration it has, emitting warnings in
                                  the logs.
  --host TEXT                     Host for the voting node API. If left unset
                                  it will look for VOTING_HOST.    If no host
                                  is found, the default value is: 0.0.0.0
  --port INTEGER                  Port for the voting node API. If left unset
                                  it will look for VOTING_PORT.    If no port
                                  is found, the default value is: 8000
  --log-level [info|debug|warn|error|trace]
                                  Sets the level for logs in the voting node.
                                  If left unset it will look for
                                  VOTING_LOG_LEVEL. If no level is found, the
                                  default value is: info
  --database-url TEXT             Sets the URL for the database. Default:
                                  postgres://localhost/CatalystEventDev
  --node-storage TEXT             Sets the path to the voting node storage
                                  directory
  --jorm-path TEXT                Path to the 'jormungandr' executable.
  --jorm-port-rest INTEGER        jormungandr REST listening port
  --jorm-port-jrpc INTEGER        jormungandr JRPC listening port
  --jorm-port-p2p INTEGER         jormungandr P2P listening port
  --jcli-path TEXT                Path to the 'jcli' executable.
  --help                          Show this message and exit.
```

Some examples:

* Start the node with custom log level, and custom paths to jormungandr and jcli executables:

```shell
voting-node start --log-level debug --jormungandr-path target/debug/jormungandr --jcli-path target/debug/jcli
```

* Start the node with default settings.

```shell
voting-node start
```
