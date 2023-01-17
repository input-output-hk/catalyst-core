# snapshot wormhole

Snapshot wormhole is a specialized Rest client API project with builtin scheduler for transfering snapshot result file from snapshot-trigger-service to vit-servicing-station service

## build

In main project folder run:

```
cd vit-testing/snapshot-wormhole
cargo build
```

and install:

`cargo install --path .`

## run

### quick start

The simplest run configuration is available by using command:

`snapshot-wormhole --config snapshot-wormhole.config one-shot`

which will perform a single job of snapshot-trigger-service -> vit-servicing-station

See [config](./configuration.md) for configuration file details.

### run modes

Two modes are available:

- one-shot - ends program after single job is done,
- schedule - run job continuously based on cron string.

#### one-shot

This mode can be helpful for debugging or testing purposes to verify if our configuration is correct and services are available.  

#### schedule

Start scheduler based on input cron string. We are using custom cron string which allows to program scheduler based on seconds.

The scheduling format is as follows:

 ```
| sec | min | hour | day of month | month | day of week | year | 
|  *  |  *  |   *  |      *       |   *   |      *      |   *  | 
```

For example, to schedule each run  per 15 minutes starting from now:

```
snapshot-wormhole --config wormhole-config.json schedule --cron * 4/60 * * * *" --eagerly
```

#### full list of available commands

Full list of commands is available on `snapshot-wormhole --help` command
