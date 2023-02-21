#!/usr/bin/env python3
"""
Simple program to convert a `voting-tools` snapshot into the format required by SVE 1.

SVE 1 does not support CIP-36 style weighted delegate registrations.
This tool will not accept these registrations and will
display a warning for every CIP-36 weighted delegate style registration rejected.
We do not use any non-standard packages, so this script should "just work" if Python 3 is installed.
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


def convert_sve_snapshot(args: argparse.Namespace):
    """Convert a snapshot into a format supported by SVE1."""

    # make output filename
    output = args.snapshot.with_suffix(".sve" + args.snapshot.suffix)
    print(output)

    # Load json file
    snapshot = json.loads(args.snapshot.read_text())

    new_snapshot: list[dict[str, Any]] = []
    total_rejects = 0

    for registration in snapshot:
        # Check if the delegation is a simple string.
        # If so, assume its a CIP-15 registration.
        delegation = registration["delegations"]
        if isinstance(delegation, str):
            new_snapshot.append(registration)
        elif isinstance(delegation, list):
            if len(delegation) == 1:
                registration["delegations"] = delegation[0][0]
                new_snapshot.append(registration)
            else:
                print(
                    "Multiple Delegations unsupported for SVE:\n"
                    f"{json.dumps(registration, indent=4)}\n"
                )
                total_rejects += 1
        else:
            print(
                "Unknown Registration Record Format:\n"
                f"{json.dumps(registration, indent=4)}\n"
            )
            total_rejects += 1

    if total_rejects == 0:
        print("No registrations rejected.")
    else:
        print(f"{total_rejects} registrations rejected.")

    output.write_text(json.dumps(new_snapshot, indent=4))


def main() -> int:
    """Parse CLI arguments."""
    parser = argparse.ArgumentParser(
        description="Convert voting tools snapshot into SVE required format."
    )
    parser.add_argument(
        "--snapshot",
        help="Snapshot file to read.",
        required=True,
        type=is_file,
    )

    args = parser.parse_args()
    convert_sve_snapshot(args)
    return 0


if __name__ == "__main__":
    sys.exit(main())
