"""Environment variables for the voting node."""
from typing import Final

# Service settings
VOTING_HOST: Final = "VOTING_HOST"
"""Host address for the voting node API."""

VOTING_PORT: Final = "VOTING_PORT"
"""Listening port for the voting node API."""

VOTING_LOG_LEVEL: Final = "VOTING_LOG_LEVEL"
"""Log level for the voting node."""

VOTING_LOG_FORMAT: Final = "VOTING_LOG_FORMAT"
"""Log format for the voting node."""

VOTING_NODE_STORAGE: Final = "VOTING_NODE_STORAGE"
"""Path to the voting node storage."""

VOTING_NODE_ROLE: Final = "VOTING_NODE_ROLE"
"""Role which this node will assume (e.g. leader0)."""

IS_NODE_RELOADABLE: Final = "IS_NODE_RELOADABLE"
"""Set the voting node mode to 'reloadable' if set to True."""

EVENTDB_URL: Final = "EVENTDB_URL"
"""URL to the EventDB."""

JORM_PORT_REST: Final = "JORM_PORT_REST"
JORM_PORT_JRPC: Final = "JORM_PORT_JRPC"
JORM_PORT_P2P: Final = "JORM_PORT_P2P"
JORM_PATH: Final = "JORM_PATH"
"""Path to the 'jormungandr' executable."""
JCLI_PATH: Final = "JCLI_PATH"
"""Path to the 'jcli' executable."""


# Secret variables. These are used for manipulating secrets.
COMMITTEE_CRS: Final = "COMMITTEE_CRS"
"""Common Reference String used for creating committee member keys.

The same CRS is shared per committee member, per event.
"""
SECRET_SECRET: Final = "SECRET_SECRET"
"""Password used for key encryption."""
