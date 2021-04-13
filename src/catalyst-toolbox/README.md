# catalyst-toolbox
Catalyst Tools, cli's and scripts related


## Catalyst toolbox cli

Rust based CLI utility for catalyst operations.

Build with `cargo build` and run, or run with `cargo run -- PARAMS` as per the examples.

```shell
catalyst-toolbox-cli 0.1.0

USAGE:
    catalyst-toolbox-cli.exe [FLAGS] [SUBCOMMAND]

FLAGS:
        --full-version      display full version details (software version, source
                            version, targets and compiler used)
    -h, --help              Prints help information
        --source-version    display the sources version, allowing to check the
                            source's hash used to compile this executable. this
                            option is useful for scripting retrieving the logs of the
                            version of this application
    -V, --version           Prints version information

SUBCOMMANDS:
    help       Prints this message or the help of the given subcommand(s)
    rewards    Rewards related operations
```

### Supported operations

#### Calculate voters rewards

```shell
Calculate rewards for voters base on their stake

USAGE:
    catalyst-toolbox-cli.exe rewards voters [OPTIONS] --total-rewards <total-rewards>

FLAGS:
    -h, --help
            Prints help information

    -V, --version
            Prints version information


OPTIONS:
        --input <FILE_INPUT>
            the file path to the genesis file defining the block 0

            If not available the command will expect to read the configuration from the standard input.
        --output <FILE_OUTPUT>
            the file path to the block to create

            If not available the command will expect to write the block to to the standard output
        --total-rewards <total-rewards>
            Reward (in LOVELACE) to be distributed
```
