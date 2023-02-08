# Voting Node Service

This service acts as a wrapper around a `jormungandr`, interacting with the Election DB to gather the necessary so that the node can join in the network.


# Running locally

## Requirements

* git
* python3, pip
* jormungandr
* jcli

Access to postgres with Election DB.

1. Get a working copy of the repository and change into the new directory.

```shell
git clone https://github.com/input-output-hk/catalyst-core.git
cd catalyst-core
```

2. Create a python3 virtual environment
```shell
python3 -m venv .env
. .env/bin/activate
pip install --upgrade pip
```

3. Install `voting_node` package

In development mode:
```
pip install --editable ./services/voting-node/
```

In release mode:
```
pip install ./services/voting-node/
```

4. Run the voting-node

A quick way to run the latest jormungandr/jcli executables is to set their paths. If the paths are omitted, the default paths are `$HOME/.cargo/bin/jormungandr` and `$HOME/.cargo/bin/jcli`. The default log level is `info` by pythonic default, but for development mode, `debug` should be preferred.

```shell
voting-node start --log-level debug --jormungandr-path target/debug/jormungandr --jcli-path target/debug/jcli
```

or run with the default settings:

```shell
voting-node start
```

which is equivalent to

```shell
voting-node start --log-level info --jormungandr-path $HOME/.cargo/bin/jormungandr --jcli-path $HOME/.cargo/bin/jcli
```
