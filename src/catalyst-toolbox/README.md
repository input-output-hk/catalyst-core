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
    catalyst-toolbox-cli rewards voters [OPTIONS] --total-rewards <total-rewards>

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
    catalyst-toolbox-cli push-notification send <SUBCOMMAND>

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
    catalyst-toolbox-cli push-notification send from-args [FLAGS] [OPTIONS] --access-token <acces
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

###### Plain string
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
    catalyst-toolbox-cli.exe push-notification send from-json [OPTIONS] [content-path]

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