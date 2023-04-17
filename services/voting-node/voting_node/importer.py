"""External data importer.

Import data from external services used for voting.


Requirements:

* `ideascale-importer` utility used to import external data.
* A file named `ideascale-importer-config.json` on the current path (TODO: this needs to be set as an envvar).

This module requires the following environment variables to be set:

* `EVENTDB_URL`

Specific to Ideascale

* `IDEASCALE_API_TOKEN`
* `IDEASCALE_CAMPAIGN_GROUP`
* `IDEASCALE_STAGE_ID`
* `IDEASCALE_API_URL`

Specific to Snapshot

* SNAPSHOT_CONFIG_PATH
* SNAPSHOT_OUTPUT_DIR
* SNAPSHOT_RAW_FILE
* SNAPSHOT_DREPS_FILE

"""
import asyncio
import os
from typing import Final

from loguru import logger

from .envvar import EVENTDB_URL, VOTING_LOG_LEVEL

IDEASCALE_API_TOKEN: Final = "IDEASCALE_API_TOKEN"
IDEASCALE_CAMPAIGN_GROUP: Final = "IDEASCALE_CAMPAIGN_GROUP"
IDEASCALE_STAGE_ID: Final = "IDEASCALE_STAGE_ID"
IDEASCALE_API_URL: Final = "IDEASCALE_API_URL"

SNAPSHOT_CONFIG_PATH: Final = "SNAPSHOT_CONFIG_PATH"
"""Path to the configuration file for the `ideascale-importer snapshot` tool."""
SNAPSHOT_OUTPUT_DIR: Final = "SNAPSHOT_OUTPUT_DIR"
"""Output directory for snapshot data. This directory MUST exist."""
SNAPSHOT_RAW_FILE: Final = "SNAPSHOT_RAW_FILE"
"""Optional raw snapshot data. When set, the snapshot-tool is not used."""
SNAPSHOT_DREPS_FILE: Final = "SNAPSHOT_DREPS_FILE"
"""Optional DReps data."""


class ExternalDataImporter:
    """Importer of external data."""

    async def ideascale_import_all(self, event_id: int):
        """Run 'ideascale-importer ideascale import-all <ARGS..>' as a subprocess."""
        logger.info(f"Running ideascale for event {event_id}")
        proc = await asyncio.create_subprocess_exec(
            "ideascale-importer",
            "ideascale",
            "import-all",
            "--api-token",
            os.environ[IDEASCALE_API_TOKEN],
            "--database-url",
            os.environ[EVENTDB_URL],
            "--event-id",
            f"{event_id}",
            "--campaign-group-id",
            os.environ[IDEASCALE_CAMPAIGN_GROUP],
            "--stage-id",
            os.environ[IDEASCALE_STAGE_ID],
            "--ideascale-api-url",
            os.environ[IDEASCALE_API_URL],
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
        """Run 'ideascale-importer snapshot import <ARGS..>' as a subprocess."""
        logger.info(f"Importing snapshot data for event {event_id}")
        proc = await asyncio.create_subprocess_exec(
            "ideascale-importer",
            "snapshot",
            "import",
            "--config-path",
            os.environ[SNAPSHOT_CONFIG_PATH],
            "--database-url",
            os.environ[EVENTDB_URL],
            "--event-id",
            f"{event_id}",
            "--raw-snapshot-file",
            os.environ[SNAPSHOT_RAW_FILE],
            #"--dreps-file",
            #os.environ[SNAPSHOT_DREPS_FILE],
            "--output-dir",
            os.environ[SNAPSHOT_OUTPUT_DIR],
            "--log-level",
            os.environ[VOTING_LOG_LEVEL],
            "--log-format",
            "text",
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
