from urllib.parse import urljoin

import requests
import ijson
import json
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
            if self.index.exists(fragment["fragment_id"]):
                # TODO: Possibly break here if we know that the fragments are ordered
                continue

            print(json.dumps(fragment))
            self.index.insert(fragment["fragment_id"])
