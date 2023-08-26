#!/usr/bin/env python3
"""
Simple program to compare snapshots.
"""

from __future__ import annotations

import argparse
import sys
import json

from pathlib import Path
from typing import Any

def is_dir(dirpath: str | Path):
    """Check if the directory is a directory."""
    real_dir = Path(dirpath)
    if real_dir.exists() and real_dir.is_dir():
        return real_dir
    raise argparse.ArgumentTypeError(f"{dir} is not a directory.")


def is_file(filename: str):
    """Does the path exist and is it a file"""
    real_filename = Path(filename).relative_to(".")
    is_dir(real_filename.parent)
    if real_filename.is_dir():
        raise argparse.ArgumentTypeError(f"{filename} is not a file.")
    return real_filename

def normalize_snapshot(snapshot) -> dict:
    normalized={}

    if isinstance(snapshot, list):
        for rec in snapshot:
            normalized[rec["hir"]["address"]] = rec["hir"]["voting_power"]
    else:
        # legacy snapshot
        for rec in snapshot["initial"][0]["fund"]:
            normalized[rec["address"]] = rec["value"]

    return normalized


def analyze_snapshot(args: argparse.Namespace):
    """Compare snapshots.  ONLY checks if the address and voting power match."""
    snapshot = normalize_snapshot(json.loads(args.snapshot.read_text()))
    compare = normalize_snapshot(json.loads(args.compare.read_text()))

    for key in snapshot:
        if key not in compare:
            print(f"Voter Address not found {key}");
        else:
            cmp = compare.pop(key)
            snap = snapshot[key]
            if cmp == snap:
                continue
            if (cmp > snap) and int(cmp / 1000000) == snap:
                continue
            if (cmp < snap) and int(snap/ 1000000) == cmp:
                continue

            if cmp > snap:
                print(int(cmp / 1000000))
            if snap > cmp:
                print(int(snap / 1000000))
            print(f"Voter {key} {snapshot[key]} != {cmp}");

    for key in compare:
        print(f"Comparison Voter Address not found {key}");



def main() -> int:
    """Parse CLI arguments."""
    parser = argparse.ArgumentParser(
        description="Compare fully processed snapshots."
    )
    parser.add_argument(
        "--snapshot",
        help="Snapshot file to read.",
        required=True,
        type=is_file,
    )

    parser.add_argument(
        "--compare",
        help="Snapshot file to compare with.",
        required=False,
        type=is_file,
    )

    args = parser.parse_args()
    analyze_snapshot(args)
    return 0


if __name__ == "__main__":
    sys.exit(main())