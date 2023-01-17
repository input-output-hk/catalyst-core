# Registration CLI

Registration CLI is cli utility tool which help to interact with registration service.

## Build & Install

In order to build registration-service in main project folder run:

```
cd registration-service
cargo build
cargo install --path . --force
```

## Quick Start

The simplest usage example is available by using commands:

* register new job:
`snapshot-cli --endpoint https://snapshot.io job new --tag daily`

NOTE: response of the above call should return `job-id` which should be used in next call like below:

`b0b7b774-7263-4dce-a97d-c167169c8f27`

* check job id:
`snapshot-cli job status --job-id {job-id} --endpoint https://{ADDRESS}`

### full list of available commands

Full list of commands is available on `snapshot-cli --help` command.

```
snapshot-trigger-service 0.1.0

USAGE:
    snapshot-cli.exe [OPTIONS] --endpoint <endpoint> <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -e, --endpoint <endpoint>    snapshot endpoint [env: SNAPSHOT_ENDPOINT=]
    -t, --token <token>          access token, which is necessary to perform client operations [env: SNAPSHOT_TOKEN=]

SUBCOMMANDS:
    files     retrieve files from snapshot (snapshot outcome etc.)
    health    check if snapshot service is up
    help      Prints this message or the help of the given subcommand(s)
    job       job related commands
```
