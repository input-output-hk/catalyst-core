# Iapyx Load

Iapyx-load is a load cli & api project which operates on catalyst backend.

## Build & Install

In order to build iapyx-load in main project folder run:

```
cd testing/iapyx
cargo build
cargo install --path . --force
```

## Quick Start

### CLI

Iapyx-load can be used as a cli. It is capable of putting various load on catalyst backend.
Available load types:

* node-only     - Load which targets blockchain calls only
* static-only   - Load which targets static data only
* simulation    - Load with simulate real user case (both blockchain and static data in some relation)

Also `node-only` load provides two load characteristic:

* bursts        - Bursts mode. Sends votes in batches and then wait x seconds
* const        - Constant load. Sends votes with x votes per second speed

And two scenario types:

* duration      - Duration based load. Defines how much time load should run
* count         - Requests count based load. Defines how many requests load should sent in total

Simplest load configuration is to use node-only load with below parameters:

`iapyx-load node-only const count --help`

```
USAGE:
    iapyx-load.exe node-only const count [FLAGS] [OPTIONS] --requests-per-thread <count>

FLAGS:
        --debug                   Print additional information
        --help                    Prints help information
        --read-from-filename      Read pin from filename of each qr code
        --reuse-accounts-early    Update all accounts state before sending any vote
        --reuse-accounts-lazy     Update account state just before sending vote
    -h, --https                   Use https for sending fragments
    -V, --version                 Prints version information

OPTIONS:
    -a, --address <address>                        Address in format: 127.0.0.1:8000 [default: 127.0.0.1:8000]
    -n, --requests-per-thread <count>              How many requests per thread should be sent
    -c, --criterion <criterion>                    Pass criteria
    -d, --delay <delay>                            Amount of delay [miliseconds] between requests [default: 10000]
        --global-pin <global-pin>                  Global pin for all qr codes [default: 1234]
    -b, --progress-bar-mode <progress-bar-mode>
            Show progress. Available are (Monitor,Standard,None) [default: Monitor]

    -q, --qr-codes-folder <qr-codes-folder>        Qr codes source folder
    -s, --secrets-folder <secrets-folder>          Secrets source folder
        --status-pace <status-pace>                How frequent (in seconds) to print status [default: 1]
    -t, --threads <threads>                        Prints nodes related data, like stats,fragments etc [default: 3]
```

### API

Iapyx load main purpose is to serve as load api:

```
use iapyx::{NodeLoad, NodeLoadConfig};
use jortestkit::{
    load::{ConfigurationBuilder, Monitor},
    measurement::Status,
};

...

    let no_of_threads = 10;
    let no_of_wallets = 40_000;
   
    let mut qr_codes_folder = Path::new("qr-codes");

    let config = ConfigurationBuilder::duration(parameters.calculate_vote_duration())
        .thread_no(threads_no)
        .step_delay(Duration::from_millis(delay))
        .fetch_limit(250)
        .monitor(Monitor::Progress(100))
        .shutdown_grace_period(Duration::from_secs(60))
        .build();

    let load_config = NodeLoadConfig {
        batch_size,
        use_v1: false,
        config,
        criterion: Some(100),
        address: "127.0.0.1:8080".to_string(),
        qr_codes_folder: Some(qr_codes_folder),
        secrets_folder: None,
        global_pin: "".to_string(),
        reuse_accounts_lazy: false,
        reuse_accounts_early: false,
        read_pin_from_filename: true,
        use_https: false,
        debug: false,
    };

    let iapyx_load = NodeLoad::new(load_config);
    if let Some(benchmark) = iapyx_load.start().unwrap() {
        assert!(benchmark.status() == Status::Green, "too low efficiency");
    }

```

### full list of available commands

Full list of commands is available on `mjolnir --help` command.

```
mjolnir 0.1.0
Jormungandr Load CLI toolkit

USAGE:
    mjolnir.exe [FLAGS] [SUBCOMMAND]

FLAGS:
        --full-version      display full version details (software version, source version, targets and compiler used)
    -h, --help              Prints help information
        --source-version    display the sources version, allowing to check the source's hash used to compile this
                            executable. this option is useful for scripting retrieving the logs of the version of this
                            application
    -V, --version           Prints version information

SUBCOMMANDS:
    explorer    Explorer load
    fragment    Fragment load
    help        Prints this message or the help of the given subcommand(s)
    passive     Passive Nodes bootstrap
    rest        Rest load
```
