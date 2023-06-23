# Importing Snapshot Data

## Configuration

See the [example config](../snapshot-importer-example-config.json) for the available configuration fields.

## Command

In order to import snapshot data you'll need:

1. a database with the latest `event-db` migrations applied with an event row inserted
2. another database populated with dbsync
3. the snapshot_tool binary
4. the catalyst-toolbox binary

*The script uses the event information in the database to define voting power threshold and max voting power percentage.*

With that you can run:

```sh
env $(cat snapshot-importer-example-env.env | xargs) poetry run python -m ideascale_importer.app snapshot import --event-id EVENT_ROW_ID
```

Where `snapshot-importer-example-env.env` is an env file containing the CLI arguments set as environment variables (they can be passed through the CLI command as well see `--help`).

If everything went as expected you should have snapshot, voters and contributions data inserted to the database.
