from datetime import datetime
import json
from pydantic.dataclasses import dataclass
import pydantic.tools


@dataclass
class DbSyncDatabaseConfig:
    host: str
    user: str
    password: str
    db: str


@dataclass
class SnapshotToolConfig:
    path: str
    max_time: datetime


@dataclass
class CatalystToolboxConfig:
    path: str


@dataclass
class GvcConfig:
    api_url: str


@dataclass
class Config:
    dbsync_database: DbSyncDatabaseConfig
    snapshot_tool: SnapshotToolConfig
    catalyst_toolbox: CatalystToolboxConfig
    gvc: GvcConfig


def from_json_file(path: str) -> Config:
    """
    Loads configuration from a JSON file.
    """

    with open(path) as f:
        return pydantic.tools.parse_obj_as(Config, json.load(f))
