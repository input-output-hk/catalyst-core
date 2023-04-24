"""Snapshot importer."""

import asyncio
import brotli
from datetime import datetime
import json
import os
from typing import Dict, List, Tuple, Optional
from loguru import logger
from pydantic.dataclasses import dataclass
import pydantic.tools

from ideascale_importer.gvc import Drep, Client as GvcClient
import ideascale_importer.db
from ideascale_importer.db import models
from ideascale_importer.utils import run_cmd


@dataclass
class DbSyncDatabaseConfig:
    """Configuration for the database containing data from dbsync."""

    host: str
    user: str
    password: str
    db: str


@dataclass
class SnapshotToolConfig:
    """Configuration for snapshot_tool."""

    path: str

@dataclass
class CatalystToolboxConfig:
    """Configuration for catalyst-toolbox."""

    path: str


@dataclass
class GvcConfig:
    """Configuration for GVC API."""

    api_url: str


@dataclass
class Config:
    """Configuration for the snapshot importer."""

    dbsync_database: DbSyncDatabaseConfig
    snapshot_tool: SnapshotToolConfig
    catalyst_toolbox: CatalystToolboxConfig
    gvc: GvcConfig

    @staticmethod
    def from_json_file(path: str) -> "Config":
        """Load configuration from a JSON file."""
        with open(path) as f:
            return pydantic.tools.parse_obj_as(Config, json.load(f))


@dataclass
class Contribution:
    """Represents a voting power contribution."""

    reward_address: str
    stake_public_key: str
    value: int


@dataclass
class HIR:
    """Represents a HIR."""

    voting_group: str
    voting_key: str
    voting_power: int


@dataclass
class SnapshotProcessedEntry:
    """Represents a processed entry from snapshot_tool."""

    contributions: List[Contribution]
    hir: HIR


@dataclass
class Registration:
    """Represents a voter registration."""

    delegations: List[Tuple[str, int]] | str
    rewards_address: str
    stake_public_key: str
    voting_power: int
    voting_purpose: Optional[int]


class OutputDirectoryDoesNotExist(Exception):
    """Raised when the importer's output directory does not exist."""

    def __init__(self, output_dir: str):
        """Initialize the exception."""
        self.output_dir = output_dir

    def __str__(self):
        """Return a string representation of the exception."""
        return f"Output directory does not exist: {self.output_dir}"


class FetchParametersFailed(Exception):
    """Raised when fetching parameters from the database fails."""

    ...


class RunCatalystToolboxSnapshotFailed(Exception):
    """Raised when running catalyst-toolbox snapshot fails."""

    ...


class WriteDbDataFailed(Exception):
    """Raised when writing data to the database fails."""

    ...


class FinalSnapshotAlreadyPresent(Exception):
    ...


class Importer:
    """Snapshot importer."""

    def __init__(
        self,
        config_path: str,
        database_url: str,
        event_id: int,
        output_dir: str,
        network_id: str,
        raw_snapshot_file: Optional[str] = None,
        dreps_file: Optional[str] = None,
    ):
        """Initialize the importer."""
        self.config = Config.from_json_file(config_path)
        self.database_url = database_url
        self.event_id = event_id
        self.dreps: List[Drep] = []
        self.lastest_block_time: Optional[datetime] = None
        self.latest_block_slot_no: Optional[int] = None
        self.registration_snapshot_slot: Optional[int] = None
        self.registration_snapshot_block_time: Optional[datetime] = None
        self.registration_snapshot_time: Optional[datetime] = None
        self.snapshot_start_time: Optional[datetime] = None
        self.min_stake_threshold: Optional[int] = None
        self.voting_power_cap: Optional[float] = None
        self.catalyst_toolbox_out_file = os.path.join(output_dir, "voter_groups.json")
        self.network_id = network_id

        if raw_snapshot_file is None:
            self.raw_snapshot_tool_file = os.path.join(output_dir, "snapshot_tool_out.json")
            self.skip_snapshot_tool_execution = False
        else:
            self.raw_snapshot_tool_file = raw_snapshot_file
            self.skip_snapshot_tool_execution = True

        self.dreps_file = dreps_file

        if not os.path.exists(output_dir):
            raise OutputDirectoryDoesNotExist(output_dir)

    async def _check_preconditions(self):
        conn = await ideascale_importer.db.connect(self.database_url)

        # Check if a final snapshot already exists
        row = await conn.fetchrow("SELECT final FROM snapshot WHERE event = $1", self.event_id)
        if row is not None and row["final"]:
            raise FinalSnapshotAlreadyPresent()

        await conn.close()

    async def _fetch_parameters(self):
        # Fetch event parameters
        conn = await ideascale_importer.db.connect(self.database_url)

        row = await conn.fetchrow(
            "SELECT registration_snapshot_time, snapshot_start, voting_power_threshold, max_voting_power_pct FROM event WHERE row_id = $1",
            self.event_id,
        )
        if row is None:
            raise FetchParametersFailed("Failed to get event parameters from the database: " f"event_id={self.event_id} not found")

        self.voting_power_cap = row["max_voting_power_pct"]
        self.min_stake_threshold = row["voting_power_threshold"]
        self.snapshot_start_time = row["snapshot_start"]
        self.registration_snapshot_time = row["registration_snapshot_time"]

        if self.snapshot_start_time is None or self.registration_snapshot_time is None:
            raise FetchParametersFailed(
                "Missing snapshot timestamps for event in the database:"
                f" snapshot_start={None if self.snapshot_start_time is None else self.snapshot_start_time.isoformat()}"
                f" registration_snapshot_time={None if self.registration_snapshot_time is None else self.registration_snapshot_time.isoformat()}"
            )

        logger.info(
            "Got event parameters",
            min_stake_threshold=self.min_stake_threshold,
            voting_power_cap=float(self.voting_power_cap),
            snapshot_start=None if self.snapshot_start_time is None else self.snapshot_start_time.isoformat(),
            registration_snapshot_time=None
            if self.registration_snapshot_time is None
            else self.registration_snapshot_time.isoformat(),
        )

        await conn.close()

        if not self.skip_snapshot_tool_execution:
            # Fetch max slot
            conn = await ideascale_importer.db.connect(
                f"postgres://{self.config.dbsync_database.user}:"
                + f"{self.config.dbsync_database.password}@{self.config.dbsync_database.host}"
                f"/{self.config.dbsync_database.db}"
            )

            # Fetch slot number and time from the block right before or equal the registration snapshot time
            row = await conn.fetchrow(
                "SELECT slot_no, time FROM block WHERE time <= $1 AND slot_no IS NOT NULL ORDER BY slot_no DESC LIMIT 1",
                self.registration_snapshot_time,
            )
            if row is None:
                raise FetchParametersFailed(
                    "Failed to get registration snapshot block data from db_sync database: " "no data returned by the query"
                )

            self.registration_snapshot_slot = row["slot_no"]
            self.registration_snapshot_block_time = row["time"]
            logger.info(
                "Got registration snapshot block data",
                slot_no=self.registration_snapshot_slot,
                block_time=self.registration_snapshot_block_time.isoformat(),
            )

            row = await conn.fetchrow(
                "SELECT slot_no, time FROM block WHERE slot_no IS NOT NULL ORDER BY slot_no DESC LIMIT 1",
            )
            if row is None:
                raise FetchParametersFailed(
                    "Failed to get latest block time and slot number from db_sync database:" "no data returned by the query"
                )

            self.latest_block_slot_no = row["slot_no"]
            self.lastest_block_time = row["time"]
            logger.info(
                "Got latest block data",
                time=self.lastest_block_time.isoformat(),
                slot_no=self.latest_block_slot_no,
            )

            await conn.close()
        else:
            logger.info("Skipping querying max slot parameter")

    async def _fetch_gvc_dreps_list(self):
        logger.info("Fetching drep list from GVC")

        gvc_client = GvcClient(self.config.gvc.api_url)

        try:
            self.dreps = await gvc_client.dreps()
        except Exception:
            logger.error("Failed to get dreps, using drep cache")
            self.dreps = []

    async def _run_snapshot_tool(self):
        snapshot_tool_cmd = (
            f"{self.config.snapshot_tool.path}"
            f" --db-user {self.config.dbsync_database.user}"
            f" --db-pass {self.config.dbsync_database.password}"
            f" --db-host {self.config.dbsync_database.host}"
            f" --db {self.config.dbsync_database.db}"
            f" --min-slot 0 --max-slot {self.registration_snapshot_slot}"
            f" --network-id {self.network_id}"
            f" --out-file {self.raw_snapshot_tool_file}"
        )

        await run_cmd("snapshot_tool", snapshot_tool_cmd)

        # Process snapshot_tool output file to handle the case when voting_purpose is null.
        # We are setting it to 0 which is the value for Catalyst.
        with open(self.raw_snapshot_tool_file, "r") as f:
            snapshot_tool_out = json.load(f)
            for r in snapshot_tool_out:
                if r["voting_purpose"] is None:
                    r["voting_purpose"] = 0

        with open(self.raw_snapshot_tool_file, "w") as f:
            json.dump(snapshot_tool_out, f)

    async def _run_catalyst_toolbox_snapshot(self):
        # Could happen when the event in the database does not have these set
        if self.min_stake_threshold is None or self.voting_power_cap is None:
            raise RunCatalystToolboxSnapshotFailed(
                "min_stake_threshold and voting_power_cap must be set " "either as CLI arguments or in the database"
            )

        catalyst_toolbox_cmd = (
            f"{self.config.catalyst_toolbox.path} snapshot"
            f" -s {self.raw_snapshot_tool_file}"
            f" -m {self.min_stake_threshold}"
            f" -v {self.voting_power_cap}"
            f" --output-format json {self.catalyst_toolbox_out_file}"
        )

        await run_cmd("catalyst-toolbox", catalyst_toolbox_cmd)

    async def _write_db_data(self):
        with open(self.raw_snapshot_tool_file) as f:
            snapshot_tool_data_raw_json = f.read()
        with open(self.catalyst_toolbox_out_file) as f:
            catalyst_toolbox_data_raw_json = f.read()

        catalyst_toolbox_data: List[SnapshotProcessedEntry] = []
        for e in json.loads(catalyst_toolbox_data_raw_json):
            catalyst_toolbox_data.append(pydantic.tools.parse_obj_as(SnapshotProcessedEntry, e))
            await asyncio.sleep(0)

        snapshot_tool_data: List[Registration] = []
        for r in json.loads(snapshot_tool_data_raw_json):
            snapshot_tool_data.append(pydantic.tools.parse_obj_as(Registration, r))
            await asyncio.sleep(0)

        total_registered_voting_power = 0
        registration_delegation_data = {}
        for r in snapshot_tool_data:
            total_registered_voting_power += r.voting_power

            if isinstance(r.delegations, str):  # CIP15 registration
                voting_key = r.delegations
                voting_key_idx = 0
                voting_weight = 1

                registration_delegation_data[f"{r.stake_public_key}{voting_key}"] = {
                    "voting_key_idx": voting_key_idx,
                    "voting_weight": voting_weight,
                }
            elif isinstance(r.delegations, list):  # CIP36 registration
                for idx, d in enumerate(r.delegations):
                    voting_key = d[0]
                    voting_key_idx = idx
                    voting_weight = d[1]

                    registration_delegation_data[f"{r.stake_public_key}{voting_key}"] = {
                        "voting_key_idx": voting_key_idx,
                        "voting_weight": voting_weight,
                    }
            else:
                raise Exception("Invalid delegations format in registrations")

            await asyncio.sleep(0)

        is_snapshot_final = False
        should_update_final = False

        if self.lastest_block_time is not None and self.snapshot_start_time is not None:
            is_snapshot_final = self.lastest_block_time >= self.snapshot_start_time
            should_update_final = True
            logger.info(
                "Setting snapshot final flag",
                final=is_snapshot_final,
                snapshot_start=self.snapshot_start_time.isoformat(),
                latest_block_time=self.lastest_block_time.isoformat(),
            )

        compressed_snapshot_tool_data = brotli.compress(bytes(snapshot_tool_data_raw_json, "utf-8"))
        logger.debug(
            "Compressed snapshot_tool data", size=len(compressed_snapshot_tool_data), original_size=len(snapshot_tool_data_raw_json)
        )

        compressed_catalyst_toolbox_data = brotli.compress(bytes(catalyst_toolbox_data_raw_json, "utf-8"))
        logger.debug(
            "Compressed catalyst_toolbox data",
            size=len(compressed_catalyst_toolbox_data),
            original_size=len(catalyst_toolbox_data_raw_json),
        )

        compressed_dreps_data = brotli.compress(bytes(json.dumps(self.dreps), "utf-8"))
        logger.debug("Compressed DREPs data", size=len(compressed_dreps_data), original_size=len(json.dumps(self.dreps)))

        if self.lastest_block_time is None:
            raise WriteDbDataFailed("lastest_block_time not set")
        if self.registration_snapshot_time is None:
            raise WriteDbDataFailed("registration_snapshot_time not set")
        if self.registration_snapshot_slot is None:
            raise WriteDbDataFailed("registration_snapshot_slot not set")
        if self.latest_block_slot_no is None:
            raise WriteDbDataFailed("latest_block_slot_no not set")

        snapshot = models.Snapshot(
            event=self.event_id,
            as_at=self.registration_snapshot_time,
            as_at_slotno=self.registration_snapshot_slot,
            last_updated=self.lastest_block_time,
            last_updated_slotno=self.latest_block_slot_no,
            final=is_snapshot_final,
            dbsync_snapshot_cmd=os.path.basename(self.config.snapshot_tool.path),
            dbsync_snapshot_data=compressed_snapshot_tool_data,
            drep_data=compressed_dreps_data,
            catalyst_snapshot_cmd=os.path.basename(self.config.catalyst_toolbox.path),
            catalyst_snapshot_data=compressed_catalyst_toolbox_data,
        )

        voters: Dict[str, models.Voter] = {}
        contributions: List[models.Contribution] = []
        total_contributed_voting_power = 0
        total_hir_voting_power = 0

        for ctd in catalyst_toolbox_data:
            total_hir_voting_power += ctd.hir.voting_power

            for snapshot_contribution in ctd.contributions:
                total_contributed_voting_power += snapshot_contribution.value

                voting_key = ctd.hir.voting_key
                # This can be removed once it's fixed in catalyst-toolbox
                if not voting_key.startswith("0x"):
                    voting_key = "0x" + voting_key

                delegation_data = registration_delegation_data[f"{snapshot_contribution.stake_public_key}{voting_key}"]

                contribution = models.Contribution(
                    stake_public_key=snapshot_contribution.stake_public_key,
                    snapshot_id=0,
                    voting_key=voting_key,
                    voting_weight=delegation_data["voting_weight"],
                    voting_key_idx=delegation_data["voting_key_idx"],
                    value=snapshot_contribution.value,
                    voting_group=ctd.hir.voting_group,
                    reward_address=snapshot_contribution.reward_address,
                )

                voter = models.Voter(
                    voting_key=voting_key,
                    snapshot_id=0,
                    voting_group=ctd.hir.voting_group,
                    voting_power=ctd.hir.voting_power,
                )

                contributions.append(contribution)
                voters[f"{voter.voting_key}{voter.voting_group}"] = voter

            await asyncio.sleep(0)

        logger.info(
            "Done processing contributions and voters",
            total_registered_voting_power=total_registered_voting_power,
            total_contributed_voting_power=total_contributed_voting_power,
            total_hir_voting_power=total_hir_voting_power,
        )

        conn = await ideascale_importer.db.connect(self.database_url)

        async with conn.transaction():
            # Do not update the final column if we are not sure about it.
            snapshot_update_excluded_cols = ["final"]
            if should_update_final:
                snapshot_update_excluded_cols = []

            snapshot_row_id = await ideascale_importer.db.upsert(
                conn,
                snapshot,
                ["event"],
                exclude_update_cols=snapshot_update_excluded_cols,
                returning="row_id",
            )
            if snapshot_row_id is None:
                raise WriteDbDataFailed("Failed to upsert snapshot")

            for c in contributions:
                c.snapshot_id = snapshot_row_id
            for v in voters.values():
                v.snapshot_id = snapshot_row_id

            await ideascale_importer.db.delete_snapshot_data(conn, snapshot_row_id)
            await ideascale_importer.db.insert_many(conn, contributions)
            await ideascale_importer.db.insert_many(conn, list(voters.values()))

        logger.info(
            "Finished writing snapshot data to database",
            contributions_count=len(contributions),
            voters_count=len(voters.values()),
        )

    async def run(self):
        """Take a snapshot and write the data to the database."""
        await self._fetch_parameters()

        if self.dreps_file is None:
            await self._fetch_gvc_dreps_list()
        else:
            logger.info("Skipping dreps GVC API call. Reading dreps file")
            with open(self.dreps_file) as f:
                self.dreps = json.load(f)

        if self.skip_snapshot_tool_execution:
            logger.info("Skipping snapshot_tool execution. Using raw snapshot file")
        else:
            await self._run_snapshot_tool()

        await self._run_catalyst_toolbox_snapshot()
        await self._write_db_data()
