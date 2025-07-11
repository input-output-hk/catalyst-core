"""External data importer.

Import data from external services used for voting.

# Required environment variables

This module requires the following environment variables to be set:

Common to IdeaScale and DBSync snapshots:

* `EVENTDB_URL` - URL to the EventDB.

## Specific to Ideascale snapshot

* `IDEASCALE_API_TOKEN` - API token from ideascale.com.
* `IDEASCALE_API_URL` - URL for IdeaScale API.

## Specific to DBSync Snapshot

* `SNAPSHOT_CONFIG_PATH` - Path to the command configuration file.
* `SNAPSHOT_OUTPUT_DIR`- Path to directory where DBSync snapshot output is written.
* `SNAPSHOT_NETWORK_IDS` - Defines 'mainnet' and/or 'testnet'.
* `SNAPSHOT_TOOL_PATH` - Path to the snapshot_tool executable (optional).
* `CATALYST_TOOLBOX_PATH` - Path to the catalyst-toolbox executable (optional).

"""

import asyncio
import os
from datetime import datetime

from ideascale_importer.ideascale.importer import Importer as IdeascaleImporter
from ideascale_importer.snapshot_importer import Importer as DBSyncImporter, SSHConfig as SnapshotToolSSHConfig
from loguru import logger
from pydantic import BaseModel
from prometheus_client import Gauge, Counter, Histogram

# Prometheus gauge metric to track remaining snapshot intervals
snapshot_intervals_remaining = Gauge(
    "snapshot_intervals_remaining", "Number of remaining snapshot intervals before snapshot start time", ["event_id"]
)

# Prometheus gauge metric to track the Unix timestamp of the next snapshot time
next_snapshot_timestamp = Gauge("next_snapshot_timestamp", "Unix timestamp of the next scheduled snapshot time", ["event_id"])

# Prometheus counter metrics to track snapshot operations
snapshots_taken_total = Counter("snapshots_taken_total", "Total number of snapshots attempted", ["event_id"])
snapshots_completed_total = Counter("snapshots_completed_total", "Total number of snapshots completed successfully", ["event_id"])
snapshots_failed_total = Counter("snapshots_failed_total", "Total number of snapshots that failed", ["event_id"])

# Prometheus histogram metric to track snapshot duration
snapshot_duration_seconds = Histogram(
    "snapshot_duration_seconds",
    "Time taken to complete snapshots",
    ["event_id"],
    buckets=[60, 120, 300, 600, 900, 1200, 1800, 2400, 3000, 3600, 4500, 5400, 6300, 7200],  # 1min to 2hr
)


class ExternalDataImporter:
    """Importer of external data."""

    async def ideascale_import_all(self, event_id: int):
        """Run 'ideascale-importer ideascale import-all <ARGS..>' as a subprocess.

        This command requires the following environment variables to work:

        * `EVENTDB_URL` sets `--database-url`.
        * `IDEASCALE_API_TOKEN` sets `--api-token`.
        * `IDEASCALE_API_URL` sets `--ideascale-api-url`.
        """
        logger.info(f"Running ideascale for event {event_id}")
        importer = IdeascaleImporter(
            api_token=os.environ["IDEASCALE_API_TOKEN"],
            database_url=os.environ["EVENTDB_URL"],
            event_id=event_id,
            proposals_scores_csv_path=None,
            ideascale_api_url=os.environ["IDEASCALE_API_URL"],
        )
        try:
            await importer.connect()
            await importer.run()
            logger.debug("ideascale importer has finished")
        except Exception as e:
            raise Exception(f"ideascale import error: {e}") from e

    async def snapshot_import(self, event_id: int):
        """Run 'ideascale-importer snapshot import <ARGS..>' as a subprocess.

        This command requires the following environment variables to work:

        * `EVENTDB_URL` sets `--eventdb-url`.
        * `SNAPSHOT_OUTPUT_DIR` sets `--output-dir`.
        * `SNAPSHOT_NETWORK_IDS` sets `--network-ids`.
        * `SNAPSHOT_TOOL_PATH` sets `--snapshot-tool-path` (optional).
        * `CATALYST_TOOLBOX_PATH` sets `--catalyst-toolbox-path` (optional).

        To run snapshot_tool through SSH then the following are required:
        * `SNAPSHOT_TOOL_SSH`
        * `SSH_SNAPSHOT_TOOL_PATH`
        * `SSH_SNAPSHOT_TOOL_OUTPUT_DIR`
        * `SSH_SNAPSHOT_TOOL_KEYFILE`
        * `SSH_SNAPSHOT_TOOL_DESTINATION`
        """
        # Parse network IDs from the env var using the
        # same format that the DBSync snapshot importer CLI expects.
        network_ids = [id.strip() for id in os.environ["SNAPSHOT_NETWORK_IDS"].split(" ")]

        if os.getenv("SNAPSHOT_TOOL_SSH") is not None:
            snapshot_tool_path = os.getenv("SSH_SNAPSHOT_TOOL_PATH")
            snapshot_tool_out_dir = os.getenv("SSH_SNAPSHOT_TOOL_OUTPUT_DIR")
            keyfile_path = os.getenv("SSH_SNAPSHOT_TOOL_KEYFILE")
            destination = os.getenv("SSH_SNAPSHOT_TOOL_DESTINATION")

            if (
                snapshot_tool_path is not None
                and snapshot_tool_out_dir is not None
                and keyfile_path is not None
                and destination is not None
            ):
                ssh_config = SnapshotToolSSHConfig(
                    keyfile_path=keyfile_path,
                    destination=destination,
                    snapshot_tool_path=snapshot_tool_path,
                    snapshot_tool_output_dir=snapshot_tool_out_dir,
                )
            else:
                raise Exception(
                    "SSH_SNAPSHOT_TOOL_PATH, SSH_SNAPSHOT_TOOL_OUTPUT_DIR, "
                    "SSH_SNAPSHOT_TOOL_OUTPUT_DIR and SSH_SNAPSHOT_TOOL_DESTINATION "
                    "are all required when SNAPSHOT_TOOL_SSH is set"
                )
        else:
            ssh_config = None

        logger.info(f"Importing snapshot data for event {event_id}")
        importer = DBSyncImporter(
            eventdb_url=os.environ["EVENTDB_URL"],
            event_id=event_id,
            output_dir=os.environ["SNAPSHOT_OUTPUT_DIR"],
            network_ids=network_ids,
            snapshot_tool_path=os.environ.get("SNAPSHOT_TOOL_PATH", "snapshot_tool"),
            catalyst_toolbox_path=os.environ.get("CATALYST_TOOLBOX_PATH", "catalyst-toolbox"),
            ssh_config=ssh_config,
        )
        try:
            await importer.run()
            logger.debug("dbsync importer has finished")
        except Exception as e:
            raise Exception(f"dbsync importer error: {e}") from e


class SnapshotRunner(BaseModel):
    """Run snapshots from DBSync and IdeaScale."""

    registration_snapshot_time: datetime
    snapshot_start: datetime

    def snapshot_start_has_passed(self) -> bool:
        """Check if the current time is after the snapshot start time.

        :return: a boolean indicating whether the snapshot start time has passed.
        """
        now = datetime.utcnow()
        return now > self.snapshot_start

    def _remaining_intervals_n_seconds_to_next_snapshot(self, current_time: datetime, interval: int) -> tuple[int, int]:
        """Calculates the remaining number of intervals and seconds until the next snapshot.

        :param current_time: The current datetime.
        :type current_time: datetime
        :param interval: The interval in seconds.
        :type interval: int
        :return: A tuple containing the number of intervals until the next snapshot start and the number of seconds until the next interval.
        :rtype: Tuple[int, int]
        """
        delta = self.snapshot_start - min(current_time, self.snapshot_start)
        delta_seconds = int(abs(delta.total_seconds()))
        # calculate the number of intervals until the snapshot start time
        num_intervals = int(delta_seconds / interval)
        # sleep for the remaining time until the next interval
        time_til_next: int = delta_seconds % interval
        return num_intervals, time_til_next

    async def _ideascale_snapshot(self, event_id: int) -> None:
        """Call the 'ideascale-importer ideascale import-all <ARGS..>' command."""
        importer = ExternalDataImporter()
        await importer.ideascale_import_all(event_id)
        # raise Exception("ideascale import is DISABLED. Skipping...")

    async def _dbsync_snapshot(self, event_id: int) -> None:
        """Call the 'ideascale-importer snapshot import <ARGS..>' command."""
        importer = ExternalDataImporter()
        await importer.snapshot_import(event_id)

    async def take_snapshots(self, event_id: int) -> None:
        """Takes snapshots at regular intervals using ExternalDataImporter.

        Args:
        ----
            event_id (int): The ID of the event to take snapshots for.

        Returns:
        -------
            None
        """
        # Check if snapshot start time has passed
        if self.snapshot_start_has_passed():
            logger.info("Snapshot has become stable. Skipping...")
            # Set metrics to 0 when snapshots are skipped
            snapshot_intervals_remaining.labels(event_id=event_id).set(0)
            next_snapshot_timestamp.labels(event_id=event_id).set(0)
            return

        # Take snapshots at regular intervals
        while True:
            interval = int(os.getenv("SNAPSHOT_INTERVAL_SECONDS", 1800))
            current_time = datetime.utcnow()
            num_intervals, secs_to_sleep = self._remaining_intervals_n_seconds_to_next_snapshot(current_time, interval)

            # Calculate the next snapshot timestamp
            next_snapshot_time = current_time.timestamp() + secs_to_sleep

            # Update Prometheus gauge metrics
            snapshot_intervals_remaining.labels(event_id=event_id).set(num_intervals)
            next_snapshot_timestamp.labels(event_id=event_id).set(next_snapshot_time)

            logger.info(f"{num_intervals + 1} snapshots remaining. Next snapshot is in {secs_to_sleep} seconds...")
            # Wait for the next snapshot interval
            await asyncio.sleep(secs_to_sleep)

            # Take snapshot
            logger.info("Taking snapshot now")

            # Track DBSync snapshot
            snapshots_taken_total.labels(event_id=event_id).inc()
            try:
                logger.debug("|---> Starting DBSync snapshot now")
                start_time = datetime.utcnow()

                await self._dbsync_snapshot(event_id)

                end_time = datetime.utcnow()
                duration = (end_time - start_time).total_seconds()
                snapshot_duration_seconds.labels(event_id=event_id).observe(duration)
                snapshots_completed_total.labels(event_id=event_id).inc()
            except Exception:
                snapshots_failed_total.labels(event_id=event_id).inc()
                raise  # Re-raise the exception for parent code to handle

            logger.debug("|---> Starting IdeasScale snapshot now")
            await self._ideascale_snapshot(event_id)

            if num_intervals > 0:
                await asyncio.sleep(0)
                continue
            else:
                # Set metrics to 0 when snapshots are complete
                snapshot_intervals_remaining.labels(event_id=event_id).set(0)
                next_snapshot_timestamp.labels(event_id=event_id).set(0)
                break
