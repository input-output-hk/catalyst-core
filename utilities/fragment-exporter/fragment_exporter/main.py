import argparse
import os
import sys
import time

from loguru import logger

from .exporter import Exporter
from .index import Index
from .node import Node


def main():
    logger.remove()
    logger.add(sys.stderr, serialize=True)
    parser = argparse.ArgumentParser(description="Exports node fragment logs to stdout")
    parser.add_argument(
        "--url",
        type=str,
        default=os.environ.get("FRAGMENT_EXPORTER_URL"),
        help="URL of the node to export from",
        required=not bool(os.environ.get("FRAGMENT_EXPORTER_URL")),
    )
    parser.add_argument(
        "--index-path",
        type=str,
        default=os.environ.get("FRAGMENT_EXPORTER_INDEX_PATH"),
        help="A local path to a directory where the index file will be stored",
        required=not bool(os.environ.get("FRAGMENT_EXPORTER_INDEX_PATH")),
    )
    parser.add_argument(
        "--interval",
        type=int,
        default=int(os.environ.get("FRAGMENT_EXPORTER_INTERVAL", 3600)),
        help="Interval in seconds between consecutive export runs",
    )

    args = parser.parse_args()
    if not os.path.exists(args.index_path):
        logger.error(f"Path {args.index_path} does not exist")
        sys.exit(1)

    node = Node(args.url)
    index = Index(args.index_path)
    exporter = Exporter(args.url, index, node)

    while True:
        logger.info("Entering main control loop")

        try:
            logger.info("Running exporter")
            exporter.run()
        except Exception as e:
            logger.error(f"Error encountered during export: {e}")
        finally:
            logger.info(f"Sleeping for {args.interval} seconds")
            time.sleep(args.interval)


if __name__ == "__main__":
    main()
