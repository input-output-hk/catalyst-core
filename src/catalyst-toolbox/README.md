# catalyst-toolbox
Catalyst Tools, cli's and scripts related


## Catalyst toolbox cli

Rust based CLI utility for catalyst operations.

Build the tool binary with `cargo build`.
The binary can be installed with `cargo install`, or run in the tree with
`cargo run`.

Run `catalyst-toolbox --help` (or `cargo run -- --help`) to see the list of
available commands. Detailed descriptions of commands are provided below.

### Supported operations

#### Calculate voters rewards

```shell
Calculate rewards for voters base on their stake

USAGE:
    catalyst-toolbox rewards voters [OPTIONS] --total-rewards <total-rewards>

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

#### Send push notification through Pushwoosh API
You can send a push notification directly from `catalyst-toolbox-cli` with:

```shell
USAGE:
    catalyst-toolbox push send <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    from-args    Push a notification with setup taken from arguments
    from-json    Push an already built notification from a json object
    help         Prints this message or the help of the given subcommand(s)
```

There are two main subcommands for sending such notifications. The difference between them is the input type. 
One (`from-args`) derives the notification required data from cli arguments meanwhile the other (`from-json`) takes 
a preloaded json file as input.

##### from-args

```shell
USAGE:
    catalyst-toolbox push send from-args [FLAGS] [OPTIONS] --access-token <acces
s-token> --application <application> [content-path]

FLAGS:
    -h, --help                     Prints help information
        --ignore-user-timezones    Ignore user timezones when sending a message
    -V, --version                  Prints version information

OPTIONS:
        --access-token <access-token>
        --api-url <api-url>               [default: https://cp.pushwoosh.com/json/1.3/]
        --application <application>      Pushwoosh application code where message will be send
        --campaign <campaign>            Select an specific campaign to send the message to
        --filter <filter>                Filter options as described by pushwhoosh API
        --send-date <send-date>          Date and time to send notification of format  "Y-m-d H:M"
        --timezone <timezone>            Timezone of send date, for example "America/New_York"

ARGS:
    <content-path>    Path to file with notification message, if not provided will be read from
                      the stdin
```

The content file can have two types of content, a plain string or a multilanguage one.

###### Plain string
Should be a message surrounded by `"`, json style. For example:

```json
"Hello pushwoosh app!"
```

###### Multi-language content
A json style object with international language code as keys and message as value:

```json
{  
    "en": "Hello!",
    "es": "¡Hola!",
    "de": "Hallo!"
}
```


##### from-json

```shell
USAGE:
    catalyst-toolbox push send from-json [OPTIONS] [content-path]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --api-url <api-url>    Pushwoosh API url [default: https://cp.pushwoosh.com/json/1.3/]

ARGS:
    <content-path>    Path to file with notification message, if not provided will be read from
                      the stdin
```

##### Notification json format
The notification json must include all mandatory fields or request would fail. A minimal example:
```json
{
    "auth": "z2CjBa...OTbWox",
    "application": "FFFFF-00000",
    "notifications": [
        {
            "send_date": "now",
            "content": {
                "es": "Hola!",
                "en": "Hi!"
            },
            "ignore_user_timezones": false
        }
    ]
}
```
Required fields:

* `auth`: Pushwoosh API token
* `application`: Pushwoosh application code
* `notifications`: Array of notification configuration objects. Should contain at least 1 item.

notification fields:

* Required:
    * `send_date`: Either `"now"` or a datetime with format `"Y-m-d H:M"`.
    * `content`: Either a plain mesasge (`"Hello app!"`) or a multilanguage object as explained above.
    * `ignore_use_timezones`: A boolean indicating if timezones will be avoided.
* Optionals:
    * `timezone`: Timezone of the provided `send_date`. [Available timezones](https://www.php.net/manual/en/timezones.php).
    * `campaign`: Campaign name for filtering push. Should exist in pushwoosh app configuration.
    * `filter`: Filter name string. Should exist in pushwoosh app configuration. As described in [pushwoosh documentation](https://docs.pushwoosh.com/platform-docs/api-reference/messages/api-prerequisites#filter)

#### Recover tally from permanent logs

A stopgap vote tallying scheme processing ballot transactions from node's
persistent fragment logs with relaxed transaction consistency checks,
to compensate for Catalyst app's occasionally erroneous use of spending counters
which results in user's vote transactions rejected by the blockchain.

This command processes the votes as if they were submitted with correctly
incrementing spending counters in order of submission. See
[this document](./doc/tally-recovery.md) for details.

Usage:

```shell
catalyst-toolbox recover tally --block0-path block0.bin --logs-path ./logs/fragments
```

#### Logs
There are a set of utilities to work with catalyst related logs

```shell
Download, compare and get stats from sentry and persistent fragment logs

USAGE:
    catalyst-toolbox logs <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    compare    Compare Sentry and Persistent fragment logs
    help       Prints this message or the help of the given subcommand(s)
    sentry     Operate over sentry logs
```

#### Ideascale import

Import ideascale data needed to initialize vit-servicing-station database

```shell
catalyst-toolbox-ideascale-import 0.2.0

USAGE:
    catalyst-toolbox ideascale import [OPTIONS] --api-token <api-token> --chain-vote-type <chain-vote-type> --fund <fund> --fund-goal <fund-goal> --output-dir <output-dir> --tags <tags> --threshold <threshold>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --api-token <api-token>                ideascale API token [env: IDEASCALE_API_TOKEN=]
        --chain-vote-type <chain-vote-type>    either "public" or "private"
        --fund <fund>                          Fund number id
        --fund-goal <fund-goal>                Fund goal explanation
        --output-dir <output-dir>              Path to folder where fund, challenges and proposals json files will be dumped
        --stage-label <stage-label>            Stage label: stage identifiers that links to assessments scores in ideascale [default: Assess]
        --tags <tags>                          Path to json or yaml like file containing tag configuration for ideascale custom fields
        --threshold <threshold>
```

##### Tags file

We need to provide a json like file that relates ideascale custom fields keys to our current attributes.
An example would look like:

```json
{
  "proposer_url": "website_github_repository__not_required_",
  "proposal_solution": "problem_solution",
  "proposal_brief": "challenge_brief",
  "proposal_importance": "importance",
  "proposal_goal": "describe_your_solution_to_the_problem",
  "proposal_metrics": "key_metrics_to_measure"
}
```

Where **all the keys are needed** and values are ideascale custom fields. 

#### QR code generation

Generator of voting keys for testing purposes, encrypted into QR codes.

```shell
catalyst-toolbox qr-code --input <input> --pin <pin>
```

The `--input` option value should give path to a file containing an
ed25519extended voting key in the Bech32 text format.
The PIN code used to encrypt and decrypt the key is given with the `--pin`
option.

See the [protocol document](./doc/private-key-and-qr-code.md) for the format
of the QR code payload.

#### Catalyst Funds archive tool

This is the tool intended to create an easy-to-read archive of information
related to Catalyst funds.

###### Usage

```
Dump information related to catalyst fund

USAGE:
    catalyst-toolbox.exe archive <jormungandr-database> <output-dir>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <jormungandr-database>    The path to the Jormungandr database to dump transactions from
    <output-dir>              CSV output directory
```

##### Output format

This tool outputs a set of CSV files. Each file has a name in the following
format: `vote_plan_<vote plan id>.csv`.

The set of columns depends on the type of a vote plan (private or public).

For each type the following information is included:

| Name | Description |
| --- | --- |
| `fragment_id` | The ID of a vote transaction (hex) |
| `caster` | The address of the account that casted this vote |
| `proposal` | The number of the proposal this vote was for (number) |
| `time` | The time this vote was casted in the format `epoch.slot` |
| `raw_fragment` | hex-encoded transaction |

##### Public vote plans

They carry and additional column named `choice` that carries the number of the
option the account has voted for.

##### Private vote plans

_Not yet implemented._

#### Vote check

Given a dump of the blockchain of a Jormungandr deployment, verify that the provided transactions were indeed confirmed in the chain and
the tallies of relevant vote plans were correctly computed.
This is done by replaying all transactions in a local trusted node, and thus requires Jormungandr to be installed in the system.





## Python scripts

Use an updated version of `python3` and either create a venv or just install the dependencies from the
`requirements.txt` file inside the `/scripts/python` folder. 

#### Calculate proposers rewards

Load your __venv__ and/or run with your default __python3__ `python proposers_rewards.py --help`

```shell
Usage: proposers_rewards.py [OPTIONS]

  Calculate catalyst rewards after tallying process. If all --proposals-path, 
  --active-voteplan-path and --challenges_path are provided. Then,
  data is loaded from the json files on those locations. Otherwise data is
  requested to the proper API endpoints pointed to the --vit-station-url
  option. Rewards are written into a separated file for each challenge. File
  is constructed via the --output-file. For example /out/rewards.csv with
  challenges [challenge_1, challenge_2] will generate
  /out/rewards_challenge_1.csv and /out/rewards_challenge_2.csv files.

Options:
  --conversion-factor FLOAT       [required]
  --output-file TEXT              [required]
  --approval-threshold FLOAT      [default: 0.15]
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

## License

This project is licensed under either of the following licenses:

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or
  http://opensource.org/licenses/MIT)
