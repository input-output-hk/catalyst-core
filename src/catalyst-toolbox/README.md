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

## Python scripts

Use an updated version of `python3` and either create a venv or just install the dependencies from the
`requirements.txt` file inside the `/scripts/python` folder. 

#### Calculate proposers rewards

Load your __venv__ and/or run with your default __python3__ `python proposers_rewards.py --help`

```shell
Usage: proposers_rewards.py [OPTIONS]

  Calculate catalyst rewards after tallying process. If all --proposals-
  path, --active-voteplan-path and --challenges_path are provided data is
  loaded from the json files on those locations. Otherwise data is requested
  to the proper API endpoints pointed to the --vit-station-url option.

Options:
  --conversion-factor FLOAT       [required]
  --output-file TEXT              [required]
  --threshold FLOAT               [default: 0.15]
  --output-format [csv|json]      [default: csv]
  --proposals-path TEXT
  --active-voteplan-path TEXT
  --challenges-path TEXT
  --vit-station-url TEXT          [default: https://servicing-
                                  station.vit.iohk.io]

  --install-completion [bash|zsh|fish|powershell|pwsh]
                                  Install completion for the specified shell.
  --show-completion [bash|zsh|fish|powershell|pwsh]
                                  Show completion for the specified shell, to
                                  copy it or customize the installation.

  --help                          Show this message and exit.

```