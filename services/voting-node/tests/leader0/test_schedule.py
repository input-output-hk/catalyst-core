from datetime import datetime
from pathlib import Path
import socket
import pytest
from voting_node.db import EventDb
from voting_node.models import Contribution, Event, HostInfo, Objective, Proposal, ServiceSettings, Voter, VotingGroup

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


@pytest.fixture
def mock_contributions():
    return [
        Contribution(1, "stakekey", 1, "direct", 5000, "votekey", 1),
        Contribution(2, "stakekey", 1, "rep", 5000, "votekey", 1),
        Contribution(3, "stakekey", 1, "direct", 5000, "votekey", 1),
        Contribution(4, "stakekey", 1, "rep", 5000, "votekey", 1),
        Contribution(5, "stakekey", 1, "direct", 5000, "votekey", 1),
        Contribution(6, "stakekey", 1, "direct", 5000, "votekey", 1),
    ]


@pytest.fixture
def mock_objectives():
    return [
        Objective(1, 1001, 1, "Category", "Title", "Description", False, "ADA"),
        Objective(2, 1002, 1, "Category", "Title", "Description", False, "ADA"),
        Objective(3, 1003, 1, "Category", "Title", "Description", False, "ADA"),
        Objective(4, 1004, 1, "Category", "Title", "Description", False, "ADA"),
    ]


@pytest.fixture
def mock_proposals():
    return [
        Proposal(1, 301, 1, "Title", "Summary", "Category", "publickey", 7000000, "http://url", "http://files", 1.0, "Name", "Contact", "http://proposer", "Experience"),
        Proposal(2, 302, 1, "Title", "Summary", "Category", "publickey", 7000000, "http://url", "http://files", 1.0, "Name", "Contact", "http://proposer", "Experience"),
        Proposal(3, 303, 1, "Title", "Summary", "Category", "publickey", 7000000, "http://url", "http://files", 1.0, "Name", "Contact", "http://proposer", "Experience"),
        Proposal(4, 304, 2, "Title", "Summary", "Category", "publickey", 7000000, "http://url", "http://files", 1.0, "Name", "Contact", "http://proposer", "Experience"),
        Proposal(5, 305, 2, "Title", "Summary", "Category", "publickey", 7000000, "http://url", "http://files", 1.0, "Name", "Contact", "http://proposer", "Experience"),
        Proposal(6, 306, 3, "Title", "Summary", "Category", "publickey", 7000000, "http://url", "http://files", 1.0, "Name", "Contact", "http://proposer", "Experience"),
        Proposal(7, 307, 4, "Title", "Summary", "Category", "publickey", 7000000, "http://url", "http://files", 1.0, "Name", "Contact", "http://proposer", "Experience"),
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


@pytest.fixture
def mock_db_fetch_contributions(monkeypatch, mock_contributions):
    async def mock_db_call(*args, **kwargs):
        return mock_contributions

    monkeypatch.setattr(EventDb, "fetch_contributions", mock_db_call)


@pytest.fixture
def mock_db_fetch_objectives(monkeypatch, mock_objectives):
    async def mock_db_call(*args, **kwargs):
        return mock_objectives

    monkeypatch.setattr(EventDb, "fetch_objectives", mock_db_call)


@pytest.fixture
def mock_db_fetch_proposals(monkeypatch, mock_proposals):
    async def mock_db_call(*args, **kwargs):
        return mock_proposals

    monkeypatch.setattr(EventDb, "fetch_proposals", mock_db_call)


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
    mock_event, mock_db_check_if_snapshot_is_final, mock_db_fetch_voting_groups, mock_db_fetch_voters, mock_db_fetch_contributions, mock_db_fetch_objectives, mock_db_fetch_proposals
):
    schedule = Leader0Schedule()

    schedule.node.event = mock_event

    await schedule.node_snapshot_data()
    ...
