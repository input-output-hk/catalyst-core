import json
from io import BytesIO
import tempfile
from fragment_exporter.exporter import hash, Exporter
from fragment_exporter.index import Index
from fragment_exporter.node import Node
import pytest


@pytest.fixture
def temporary_db():
    temp_dir = tempfile.TemporaryDirectory()
    yield temp_dir.name
    temp_dir.cleanup()


@pytest.fixture
def index(temporary_db):
    return Index(temporary_db)


@pytest.fixture
def mock_node(mocker):
    return mocker.Mock(spec=Node)


def get_mock_response(mocker, data):
    response = mocker.Mock()
    response.raw = BytesIO(json.dumps(data).encode())
    response.raise_for_status.return_value = None
    return response


def test_exporter_run_indexes_fragments(mocker, index, mock_node):
    mock_url = "http://mock-node.com"
    mock_data = [{"fragment_id": "dummy1"}, {"fragment_id": "dummy2"}]
    mocker.patch(
        "requests.get",
        side_effect=lambda *args, **kwargs: get_mock_response(mocker, mock_data),
    )

    mock_node.get_last_block_height.return_value = 1234

    exporter = Exporter(mock_url, index, mock_node)
    exporter.run()

    for fragment in mock_data:
        assert index.exists(hash(fragment))


def test_exporter_run_prints_only_new_fragments(mocker, index, mock_node):
    mock_url = "http://mock-node.com"
    existing_fragment = {"fragment_id": "existing_fragment"}
    new_fragment = {"fragment_id": "new_fragment"}
    mock_data = [existing_fragment, new_fragment]
    mocker.patch(
        "requests.get",
        side_effect=lambda *args, **kwargs: get_mock_response(mocker, mock_data),
    )

    mock_node.get_last_block_height.return_value = 1234

    # insert existing fragment to the index
    index.insert(hash(existing_fragment))

    print_mock = mocker.patch("builtins.print")
    exporter = Exporter(mock_url, index, mock_node)
    exporter.run()

    # check that print was called only once and with the new fragment
    print_mock.assert_called_once_with(json.dumps(new_fragment))


@pytest.mark.parametrize(
    "current_block_height,previous_block_height,should_purge",
    [
        (100, 200, True),
        (300, 200, False),
    ],
)
def test_exporter_purges_correctly(
    mocker, index, mock_node, current_block_height, previous_block_height, should_purge
):
    mock_url = "http://mock-node.com"
    mock_data = [{"fragment_id": "dummy1"}, {"fragment_id": "dummy2"}]
    mocker.patch(
        "requests.get",
        side_effect=lambda *args, **kwargs: get_mock_response(mocker, mock_data),
    )

    mock_purge = mocker.patch.object(index, "purge")

    mock_node.get_last_block_height.return_value = current_block_height
    index.set_last_block_height(previous_block_height)
    exporter = Exporter(mock_url, index, mock_node)
    exporter.run()

    if should_purge:
        mock_purge.assert_called_once()
    else:
        mock_purge.assert_not_called()
        assert index.exists(hash(mock_data[0]))
        assert index.exists(hash(mock_data[1]))
