# Registration Verify CLI

Registration Verify CLI is cli utility tool which help to interact with registration service.

## Build & Install

In order to build registration verify project in main project folder run:

```
cd registration-verify-service
cargo build
cargo install --path . --force
```

## Quick Start

The simplest usage example is available by using commands:

* register new job:
`registration-verify-cli job new  --payment-skey payment.skey --payment-vkey payment.vkey  --stake-skey stake.skey --stake-vkey stake.vkey --endpoint https://{ADDRESS}`

NOTE: response of the above call should return `job-id` which should be used in next call

NOTE: see [cardano cli guide](https://developers.cardano.org/docs/stake-pool-course/handbook/keys-addresses/) for information how to create payment and stake files.

* check job id:
`registration-cli job status --job-id {job-id} --endpoint https://{ADDRESS}`

### full list of available commands

Full list of commands is available on `registration-cli --help` command.

```
registration-service 0.1.0

USAGE:
    registration-cli.exe [OPTIONS] --endpoint <endpoint> <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -e, --endpoint <endpoint>    registration service endpoint [env: REGISTRATION_ENDPOINT=]
    -t, --token <token>          access token [env: REGISTRATION_TOKEN=]

SUBCOMMANDS:
    files     download jobs artifacts
    health    check if registration service is up
    help      Prints this message or the help of the given subcommand(s)
    job       jobs related operations
```
