from urllib.parse import urljoin

import requests
import ijson
import json
import hashlib
from .index import Index
from .node import Node


class Exporter:
    """Exports fragments from a node to stdout.

    Attributes:
        url: The URL of the node
        index: A index for storing fragments
        node: The node to export from
    """

    def __init__(self, url: str, index: Index, node: Node):
        self.url = urljoin(url, "/api/v0/fragment/logs")
        self.index = index
        self.node = node

    def run(self):
        """Runs the exporter."""
        # Purge the index if the node has been restarted
        current_block_height = self.node.get_last_block_height()
        previous_block_height = self.index.get_last_block_height()
        if current_block_height < previous_block_height:
            print("Node has been restarted. Purging index.")
            self.index.purge()

        # Update the last block height
        self.index.set_last_block_height(current_block_height)

        # Get the latest fragments
        response = requests.get(self.url, stream=True)
        response.raise_for_status()
        fragments = ijson.items(response.raw, "item")

        # Find and print new fragments
        for fragment in fragments:
            if self.index.exists(hash(fragment)):
                continue

            print(json.dumps(fragment))
            self.index.insert(hash(fragment))


def hash(fragment):
    """Returns the hash of a fragment.

    Args:
        fragment: The fragment to hash

    Returns:
        The hash of the fragment
    """
    return hashlib.sha256(json.dumps(fragment).encode("utf-8")).hexdigest()
