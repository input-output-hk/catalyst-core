from datetime import datetime
import marshmallow_dataclass


class DbSyncDatabaseConfig:
    host: str
    user: str
    password: str
    db: str


class SnapshotToolConfig:
    path: str
    max_time: datetime


class CatalystToolboxConfig:
    path: str


class GvcConfig:
    api_url: str


class Config:
    dbsync_database: DbSyncDatabaseConfig
    snapshot_tool: SnapshotToolConfig
    catalyst_toolbox: CatalystToolboxConfig
    gvc: GvcConfig


DbSyncDatabaseConfigSchema = marshmallow_dataclass.class_schema(DbSyncDatabaseConfig)
SnapshotToolConfigSchema = marshmallow_dataclass.class_schema(SnapshotToolConfig)
CatalystToolboxConfigSchema = marshmallow_dataclass.class_schema(CatalystToolboxConfig)
GvcConfigSchema = marshmallow_dataclass.class_schema(GvcConfig)
ConfigSchema = marshmallow_dataclass.class_schema(Config)


def from_json_file(path: str) -> Config:
    """
    Loads configuration from a JSON file.
    """

    with open(path) as f:
        config = ConfigSchema().loads(f.read())
        assert isinstance(config, Config)
        return config
