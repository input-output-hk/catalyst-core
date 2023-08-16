from datetime import datetime
from pathlib import Path
import socket
import pytest
from voting_node.db import EventDb
from voting_node.models import Event, ServiceSettings

from voting_node.tasks import Leader0Schedule


@pytest.fixture
def mock_event():
    return Event(
        row_id=1234,
        name="Test Event",
        description="Describe it.",
        committee_size=1,
        committee_threshold=1,
        start_time=datetime.now(),
    )


@pytest.fixture
def mock_fetch_event(monkeypatch, mock_event):
    async def mock_fetch(*args, **kwargs):
        return mock_event

    monkeypatch.setattr(EventDb, "fetch_upcoming_event", mock_fetch)


@pytest.fixture
def mock_leader0_host(monkeypatch):
    async def mock_leader0():
        return "leader0"

    monkeypatch.setattr(socket, "gethostname", mock_leader0)


@pytest.mark.asyncio
async def test_leader0_schedule_instantiates_with_defaults():
    schedule = Leader0Schedule()
    assert schedule.settings == ServiceSettings()
    assert schedule.db.db_url == schedule.settings.db_url
    assert schedule.node.storage == Path(schedule.settings.storage)
    assert schedule.current_task is None


@pytest.mark.asyncio
async def test_task_node_fetch_event(mock_fetch_event, mock_event):
    schedule = Leader0Schedule()

    await schedule.node_fetch_event()

    assert schedule.node.event == mock_event
