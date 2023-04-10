import asyncio
from datetime import datetime
import json
from loguru import logger
from pydantic.dataclasses import dataclass
import pydantic.tools
import os
from typing import Dict, List, Optional, Tuple

from . import config
import ideascale_importer.db
from ideascale_importer.db import models
from ideascale_importer.gvc.client import Client as GvcClient, Drep
from ideascale_importer.utils import run_cmd


@dataclass
class Contribution:
    reward_address: str
    stake_public_key: str
    value: int


@dataclass
class HIR:
    voting_group: str
    voting_key: str
    voting_power: int


@dataclass
class SnapshotProcessedEntry:
    contributions: List[Contribution]
    hir: HIR


@dataclass
class Registration:
    delegations: List[Tuple[str, int]] | str
    reward_address: str
    stake_public_key: str
    voting_power: int
    voting_purpose: int


class OutputDirectoryDoesNotExist(Exception):
    def __init__(self, output_dir: str):
        self.output_dir = output_dir

    def __str__(self):
        return f"Output directory does not exist: {self.output_dir}"


class FetchParametersFailed(Exception):
    ...


class RunCatalystToolboxSnapshotFailed(Exception):
    ...


class WriteDbDataFailed(Exception):
    ...


class Importer:
    def __init__(
        self,
        config_path: str,
        database_url: str,
        event_id: int,
        output_dir: str,
        raw_snapshot_file: Optional[str] = None,
        dreps_file: Optional[str] = None,
    ):
        self.config = config.from_json_file(config_path)
        self.database_url = database_url
        self.event_id = event_id
        self.dreps: List[Drep] = []
        self.snapshot_tool_max_slot: Optional[int] = None
        self.catalyst_toolbox_min_stake_threshold: Optional[int] = None
        self.catalyst_toolbox_voting_power_cap: Optional[float] = None
        self.catalyst_toolbox_out_file = os.path.join(output_dir, "voter_groups.json")

        if raw_snapshot_file is None:
            self.raw_snapshot_tool_file = os.path.join(output_dir, "snapshot_tool_out.json")
            self.skip_snapshot_tool_execution = False
        else:
            self.raw_snapshot_tool_file = raw_snapshot_file
            self.skip_snapshot_tool_execution = True

        self.dreps_file = dreps_file

        if not os.path.exists(output_dir):
            raise OutputDirectoryDoesNotExist(output_dir)

    async def _fetch_parameters(self):
        if not self.skip_snapshot_tool_execution:
            # Fetch max slot
            logger.info("Querying max slot parameter")
            conn = await ideascale_importer.db.connect(
                f"postgres://{self.config.dbsync_database.user}:"
                + f"{self.config.dbsync_database.password}@{self.config.dbsync_database.host}"
                f"/{self.config.dbsync_database.db}"
            )

            row = await conn.fetchrow(
                "SELECT slot_no FROM block WHERE time <= $1 ORDER BY time DESC LIMIT 1", self.config.snapshot_tool.max_time
            )

            if row is None:
                raise FetchParametersFailed(
                    "Failed to get max_slot parameter from db_sync database: " "no data returned by the query"
                )

            self.snapshot_tool_max_slot = row["slot_no"]
            logger.info(
                "Got max_slot for max_time",
                snapshot_tool_max_slot=self.snapshot_tool_max_slot,
                max_time=self.config.snapshot_tool.max_time.isoformat(),
            )

            await conn.close()
        else:
            logger.info("Skipping querying max slot parameter")

        # Fetch min_stake_threshold and voting_power_cap
        conn = await ideascale_importer.db.connect(self.database_url)

        row = await conn.fetchrow("SELECT voting_power_threshold, max_voting_power_pct FROM event WHERE row_id = $1", self.event_id)
        if row is None:
            raise FetchParametersFailed(
                "Failed to get min_stake_threshold and voting_power_cap parameters from the event database: "
                + f"event_id={self.event_id} not found"
            )

        self.catalyst_toolbox_voting_power_cap = row["max_voting_power_pct"]
        self.catalyst_toolbox_min_stake_threshold = row["voting_power_threshold"]

        await conn.close()

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
            f"--db-user {self.config.dbsync_database.user}"
            f" --db-pass {self.config.dbsync_database.password}"
            f" --db-host {self.config.dbsync_database.host}"
            f" --db {self.config.dbsync_database.db}"
            f" --min-slot 0 - -max-slot {self.snapshot_tool_max_slot}"
            f" --out-file {self.raw_snapshot_tool_file}"
        )

        await run_cmd("snapshot_tool", snapshot_tool_cmd)

    async def _run_catalyst_toolbox_snapshot(self):
        # Could happen when the event in the database does not have these set
        if self.catalyst_toolbox_min_stake_threshold is None or self.catalyst_toolbox_voting_power_cap is None:
            raise RunCatalystToolboxSnapshotFailed(
                "min_stake_threshold and voting_power_cap must be set " "either as CLI arguments or in the database"
            )

        catalyst_toolbox_cmd = (
            f"{self.config.catalyst_toolbox.path} snapshot"
            f" -s {self.raw_snapshot_tool_file}"
            f" -m {self.catalyst_toolbox_min_stake_threshold}"
            f" -v {self.catalyst_toolbox_voting_power_cap}"
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

        snapshot = models.Snapshot(
            event=self.event_id,
            as_at=datetime.utcnow(),
            last_updated=datetime.utcnow(),
            final=False,
            dbsync_snapshot_cmd=os.path.basename(self.config.snapshot_tool.path),
            dbsync_snapshot_data=snapshot_tool_data_raw_json,
            drep_data=json.dumps(self.dreps),
            catalyst_snapshot_cmd=os.path.basename(self.config.catalyst_toolbox.path),
            catalyst_snapshot_data=catalyst_toolbox_data_raw_json,
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
            snapshot_row_id = await ideascale_importer.db.upsert(conn, snapshot, ["event"], returning="row_id")
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

    async def import_all(self):
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
