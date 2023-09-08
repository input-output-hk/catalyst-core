from urllib.parse import urljoin
import requests


class Node:
    """Represents a Jormungandr node.

    Attributes:
        url: The URL of the node
    """

    def __init__(self, url: str):
        self.url = url

    def get_last_block_height(self) -> int:
        """Get the last block height from the node.

        Returns:
            The last block height
        """
        response = requests.get(urljoin(self.url, "/api/v0/node/stats"))
        return int(response.json()["lastBlockHeight"])
