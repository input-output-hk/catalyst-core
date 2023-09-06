import pytest
from unittest.mock import patch, Mock
from fragment_exporter.node import Node


def create_mock_response(json_data):
    """Utility function to create a mock response for the requests.get call."""
    mock_resp = Mock()
    mock_resp.json.return_value = json_data
    return mock_resp


@pytest.mark.parametrize(
    "mock_response_data,expected_height",
    [
        ({"lastBlockHeight": "1000"}, 1000),
        ({"lastBlockHeight": "2000"}, 2000),
    ],
)
def test_get_last_block_height(mock_response_data, expected_height):
    """Test get_last_block_height method of Node class."""

    with patch("requests.get", return_value=create_mock_response(mock_response_data)):
        node = Node("http://dummy.url")
        assert node.get_last_block_height() == expected_height
