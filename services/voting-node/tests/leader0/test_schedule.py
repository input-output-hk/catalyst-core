from datetime import datetime
from pathlib import Path
import socket
import pytest
from voting_node.db import EventDb
from voting_node.models import Event, HostInfo, ServiceSettings, Voter, VotingGroup

from voting_node.tasks import Leader0Schedule


# Test Fixtures


@pytest.fixture
def mock_event():
    return Event(
        row_id=1234,
        name="Test Event",
        description="Describe it.",
        committee_size=1,
        committee_threshold=1,
        start_time=datetime.now(),
        snapshot_start=datetime.now(),
    )


@pytest.fixture
def leader0_host_info(mock_event, mock_leader0_hostname):
    return HostInfo(
        hostname=mock_leader0_hostname, event=mock_event.row_id, seckey="secretkey", pubkey="publickey", netkey="netkey"
    )


@pytest.fixture
def voting_groups():
    return [VotingGroup(name="direct"), VotingGroup(name="rep")]


@pytest.fixture
def mock_voters():
    return [
        Voter(1, "votekey", 1, "direct", 5000),
        Voter(2, "votekey", 1, "rep", 5000),
        Voter(3, "votekey", 1, "direct", 5000),
        Voter(4, "votekey", 1, "rep", 5000),
        Voter(5, "votekey", 1, "direct", 5000),
        Voter(6, "votekey", 1, "direct", 5000),
    ]


## Monkeypatches


@pytest.fixture
def mock_leader0_hostname(monkeypatch):
    monkeypatch.setattr(socket, "gethostname", "leader0")


@pytest.fixture
def mock_db_fetch_upcoming_event(monkeypatch, mock_event):
    async def mock_db_call(*args, **kwargs):
        return mock_event

    monkeypatch.setattr(EventDb, "fetch_upcoming_event", mock_db_call)


@pytest.fixture
def mock_db_fetch_leader0_host_info(monkeypatch, leader0_host_info):
    async def mock_db_call(*args, **kwargs):
        return leader0_host_info

    monkeypatch.setattr(EventDb, "fetch_leader_host_info", mock_db_call)


@pytest.fixture
def mock_db_check_if_snapshot_is_final(monkeypatch):
    async def mock_db_call(*args, **kwargs):
        return True

    monkeypatch.setattr(EventDb, "check_if_snapshot_is_final", mock_db_call)


@pytest.fixture
def mock_db_fetch_voting_groups(monkeypatch, voting_groups):
    async def mock_db_call(*args, **kwargs):
        return voting_groups

    monkeypatch.setattr(EventDb, "fetch_voting_groups", mock_db_call)


@pytest.fixture
def mock_db_fetch_voters(monkeypatch, mock_voters):
    async def mock_db_call(*args, **kwargs):
        return mock_voters

    monkeypatch.setattr(EventDb, "fetch_voters", mock_db_call)


# TESTS


@pytest.mark.asyncio
async def test_leader0_schedule_instantiates_with_defaults():
    schedule = Leader0Schedule()
    assert schedule.settings == ServiceSettings()
    assert schedule.db.db_url == schedule.settings.db_url
    assert schedule.node.storage == Path(schedule.settings.storage)
    assert schedule.current_task is None


@pytest.mark.asyncio
async def test_task_node_fetch_event(mock_event, mock_db_fetch_upcoming_event):
    schedule = Leader0Schedule()

    await schedule.node_fetch_event()
    assert schedule.node.event == mock_event


@pytest.mark.asyncio
async def test_task_node_fetch_host_keys(leader0_host_info, mock_event, mock_db_fetch_leader0_host_info):
    schedule = Leader0Schedule()

    schedule.node.event = mock_event

    await schedule.node_fetch_host_keys()
    assert schedule.node.host_info == leader0_host_info


### TODO: Other tasks


@pytest.mark.asyncio
async def test_task_node_snapshot_data(
    mock_event, mock_db_check_if_snapshot_is_final, mock_db_fetch_voting_groups, mock_db_fetch_voters
):
    schedule = Leader0Schedule()

    # test_event = mock_event

    schedule.node.event = mock_event

    await schedule.node_snapshot_data()
