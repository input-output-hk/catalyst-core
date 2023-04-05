# Voting Node Service

This service acts as a wrapper around a `jormungandr`, interacting with the Event DB to gather the necessary data so that the node can join in the network.


# Running locally

## Requirements

* git
* python >= 3.10
* poetry [https://python-poetry.org/docs/#installation](https://python-poetry.org/docs/#installation)
* jormungandr
* jcli

Access to postgres with Event DB.

1. Get a working copy of the repository and change into the new directory.

```shell
git clone https://github.com/input-output-hk/catalyst-core.git
```

```shell
cd catalyst-core/services/voting-node
```

2. Initialize and enter virtual environment
```shell
poetry shell
```

3. Install `voting_node` package

```
poetry install
```

4. Run the voting-node

A quick way to run the latest jormungandr/jcli executables is to set their paths. If the paths are omitted, the default paths are `jormungandr` and `jcli`. The default log level is `info` by pythonic default, but for development mode, `debug` should be preferred.

```shell
voting-node start --log-level debug --jormungandr-path target/debug/jormungandr --jcli-path target/debug/jcli
```

or run with the default settings:

```shell
voting-node start
```

which is equivalent to

```shell
voting-node start --log-level info --jormungandr-path jormungandr --jcli-path jcli
    ```
