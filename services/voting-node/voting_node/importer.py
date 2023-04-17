"""External data importer.

Import data from external services used for voting.


Requirements:

* `ideascale-importer` utility used to import external data.
* A file named `ideascale-importer-config.json` on the current path (TODO: this needs to be set as an envvar).

This module requires the following environment variables to be set:

* `EVENTDB_URL`
* `IDEASCALE_API_TOKEN`
* `IDEASCALE_CAMPAIGN_GROUP`
* `IDEASCALE_STAGE_ID`
* `IDEASCALE_LOG_LEVEL`
* `IDEASCALE_API_URL`
"""
import asyncio
import os
from typing import Final

from loguru import logger

from .envvar import EVENTDB_URL

IDEASCALE_API_TOKEN: Final = "IDEASCALE_API_TOKEN"
IDEASCALE_CAMPAIGN_GROUP: Final = "IDEASCALE_CAMPAIGN_GROUP"
IDEASCALE_STAGE_ID: Final = "IDEASCALE_STAGE_ID"
IDEASCALE_LOG_LEVEL: Final = "IDEASCALE_LOG_LEVEL"
IDEASCALE_API_URL: Final = "IDEASCALE_API_URL"


class IdeascaleImporter:
    """Importer of external data from ideascale.com."""

    async def run(self, event_id: int):
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
