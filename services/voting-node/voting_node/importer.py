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

from datetime import datetime
from loguru import logger
from pydantic import BaseModel
from typing import Final, Tuple

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
                print(line.decode().rstrip('\n'))
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
                print(line.decode().rstrip('\n'))
            else:
                break

        returncode = await proc.wait()
        if returncode != 0:
            raise Exception("failed to run ideascale importer")
        logger.debug("ideascale importer has finished")



class SnapshotRunner(BaseModel):
    """Run snapshots from DBSync and IdeaScale."""
    registration_snapshot_time: datetime
    snapshot_start: datetime

    def snapshot_start_has_passed(self) -> bool:
        """
        Check if the current time is after the snapshot start time.

        :return: a boolean indicating whether the snapshot start time has passed.
        """
        now = datetime.utcnow()
        return now > self.snapshot_start

    def _reimaining_intervals_n_seconds_to_next_snapshot(self, current_time: datetime, interval: int) -> Tuple[int, int]:
        """
        Calculates the remaining number of intervals and seconds until the next snapshot.

        :param current_time: The current datetime.
        :type current_time: datetime
        :param interval: The interval in seconds.
        :type interval: int
        :return: A tuple containing the number of intervals until the next snapshot start and the number of seconds until the next interval.
        :rtype: Tuple[int, int]
        """
        delta = self.snapshot_start - current_time
        delta_seconds = int(abs(delta.total_seconds()))
        # calculate the number of intervals until the snapshot start time
        num_intervals = int(delta_seconds / interval)
        # sleep for the remaining time until the next interval
        time_til_next: int = delta_seconds % interval
        return num_intervals, time_til_next

    async def take_snapshots(self, event_id: int) -> None:
        """
        Takes snapshots at regular intervals using ExternalDataImporter.

        Args:
            event_id (int): The ID of the event to take snapshots for.

        Returns:
            None
        """
        # Check if snapshot start time has passed
        if self.snapshot_start_has_passed():
            logger.info("Snapshot has become stable. Skipping...")
            return

        # Initialize external data importer
        importer = ExternalDataImporter()

        # Take snapshots at regular intervals
        while True:
            interval = int(os.getenv('SNAPSHOT_INTERVAL_SECONDS', 1800))
            current_time = datetime.utcnow()
            num_intervals, secs_to_sleep = self._reimaining_intervals_n_seconds_to_next_snapshot(current_time, interval)
            if num_intervals > 0:
                # Wait for the next snapshot interval
                logger.info(f"Next snapshot is in {secs_to_sleep} seconds...")
                await asyncio.sleep(secs_to_sleep)

                # Take snapshot
                logger.info("Taking snapshot now")
                try:
                    await importer.snapshot_import(event_id)
                except Exception as e:
                    logger.error("Failed to take dbsync snapshot", exc_info=e)

                try:
                    await importer.ideascale_import_all(event_id)
                except Exception as e:
                    logger.error("Failed to take ideascale snapshot", exc_info=e)

                await asyncio.sleep(0)
                continue
            else:
                # Take final snapshot
                logger.info(f"Taking FINAL snapshot in {secs_to_sleep} seconds...")
                await asyncio.sleep(secs_to_sleep)
                logger.info("Taking FINAL snapshot now")
                try:
                    await importer.snapshot_import(event_id)
                except Exception as e:
                    logger.error("Failed to take dbsync snapshot", exc_info=e)

                try:
                    await importer.ideascale_import_all(event_id)
                except Exception as e:
                    logger.error("Failed to take ideascale snapshot", exc_info=e)

                break
