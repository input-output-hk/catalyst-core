# Importing Snapshot Data

## Configuration

See the [example config](../snapshot-importer-example-config.json) for the available configuration fields.

## Command

In order to import snapshot data you'll need:

1. a database with the latest `event-db` migrations applied with an event row inserted
3. another database populated with dbsync
4. the snapshot_tool binary
5. the catalyst-toolbox binary

*The script uses the event information in the database to define voting power threshold and max voting power percentage.*

With that you can run:

```sh
PYTHONPATH=(pwd) poetry run python ideascale_importer snapshot import --config-path PATH_TO_CONFIG_FILE --event-id EVENT_ROW_ID --database-url VITSS_DB_URL --output-dir OUTDIR_PATH
```

If everything went as expected you should have snapshot, voters and contributions data inserted to the database.
