"""External data importer.

Import data from external services used for voting.

# Required environment variables

This module requires the following environment variables to be set:

Common to IdeaScale and DBSync snapshots:

* `EVENTDB_URL` - URL to the EventDB.

## Specific to Ideascale snapshot

* `IDEASCALE_API_TOKEN` - API token from ideascale.com.
* `IDEASCALE_CAMPAIGN_GROUP` - Group ID for the IdeaScale campaign.
* `IDEASCALE_STAGE_ID` - Stage ID for IdeaScale.
* `IDEASCALE_API_URL` - URL for IdeaScale API.
* `IDEASCALE_LOG_LEVEL` - Set the log level for the importer command (optional).
* `IDEASCALE_LOG_FORMAT` - Set the log format for the importer command (optional).

## Specific to DBSync Snapshot

* `SNAPSHOT_CONFIG_PATH` - Path to the command configuration file.
* `SNAPSHOT_OUTPUT_DIR`- Path to directory where DBSync snapshot output is written.
* `SNAPSHOT_NETWORK_ID` - Defines 'mainnet' or 'testnet'.
* `SNAPSHOT_LOG_LEVEL` - Set the log level for the importer command (optional).
* `SNAPSHOT_LOG_FORMAT` - Set the log format for the importer command (optional).

"""
import asyncio
import os
from typing import Final

from loguru import logger


class ExternalDataImporter:
    """Importer of external data."""

    async def ideascale_import_all(self, event_id: int):
        """Run 'ideascale-importer ideascale import-all <ARGS..>' as a subprocess.

        This command requires the following environment variables to work:

        * `EVENTDB_URL` sets `--database-url`.
        * `IDEASCALE_API_TOKEN` sets `--api-token`.
        * `IDEASCALE_CAMPAIGN_GROUP` sets `--campaing-group-id`.
        * `IDEASCALE_STAGE_ID` sets `--stage-id`.
        * `IDEASCALE_API_URL` sets `--ideascale-api-url`.
        * `IDEASCALE_LOG_LEVEL` sets `--log-level` (optional).
        * `IDEASCALE_LOG_FORMAT` sets `--log-format` (optional).
        """
        logger.info(f"Running ideascale for event {event_id}")
        proc = await asyncio.create_subprocess_exec(
            "ideascale-importer",
            "ideascale",
            "import-all",
            "--event-id",
            f"{event_id}",
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.STDOUT,
        )

        # checks that there is stdout
        while proc.stdout is not None:
            line = await proc.stdout.readline()
            if line:
                print(line.decode())
            else:
                break

        returncode = await proc.wait()
        if returncode != 0:
            raise Exception("failed to run ideascale importer")
        logger.debug("ideascale importer has finished")

    async def snapshot_import(self, event_id: int):
        """Run 'ideascale-importer snapshot import <ARGS..>' as a subprocess.

        This command requires the following environment variables to work:

        * `EVENTDB_URL` sets `--database-url`.
        * `SNAPSHOT_CONFIG_PATH` sets `--config-path`.
        * `SNAPSHOT_OUTPUT_DIR` sets `--output-dir`.
        * `SNAPSHOT_NETWORK_ID` sets `--network-id`.
        * `SNAPSHOT_LOG_LEVEL` sets `--log-level` (optional).
        * `SNAPSHOT_LOG_FORMAT` sets `--log-format` (optional).
        """
        logger.info(f"Importing snapshot data for event {event_id}")
        proc = await asyncio.create_subprocess_exec(
            "ideascale-importer",
            "snapshot",
            "import",
            "--event-id",
            f"{event_id}",
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.STDOUT,
        )

        # checks that there is stdout
        while proc.stdout is not None:
            line = await proc.stdout.readline()
            if line:
                print(line.decode())
            else:
                break

        returncode = await proc.wait()
        if returncode != 0:
            raise Exception("failed to run ideascale importer")
        logger.debug("ideascale importer has finished")
