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


def analyze_snapshot(args: argparse.Namespace):
    """Convert a snapshot into a format supported by SVE1."""

    # make output filename
    output = args.snapshot.with_suffix(".sve" + args.snapshot.suffix)
    print(output)

    # Load json file
    snapshot = json.loads(args.snapshot.read_text())
    snapshot_index : dict[str, Any] = {}

    cip_15_snapshot: list[dict[str, Any]] = []
    cip_36_single: list[dict[str, Any]] = []
    cip_36_multi: list[dict[str, Any]] = []

    stake_types: dict[str, Any] = {}

    total_rejects = 0

    for registration in snapshot:
        # Index the registrations
        stake_pub_key = registration["stake_public_key"]
        snapshot_index[stake_pub_key] = registration

        # Check if the delegation is a simple string.
        # If so, assume its a CIP-15 registration.
        delegation = registration["delegations"]

        if isinstance(delegation, str):
            cip_15_snapshot.append(registration)
        elif isinstance(delegation, list):
            if len(delegation) == 1:
                cip_36_single.append(registration)
            else:
                cip_36_multi.append(registration)
        else:
            print(
                "Unknown Registration Record Format:\n"
                f"{json.dumps(registration, indent=4)}\n"
            )
            total_rejects += 1

        stake_type = registration["stake_public_key"][2]
        if stake_type in stake_types:
            stake_types[stake_type] += 1
        else:
            stake_types[stake_type] = 1

    # Load Errors if we can.
    registration_errors : dict[str, Any] = {}
    no_stake_key = 0
    unknown_errors = 0
    missing_registration_metadata = 0

    if args.errors is not None:
        raw_registration_errors = json.loads(args.errors.read_text())
        # Index the registration errors by their stake key.
        for error in raw_registration_errors:
            if "registration" not in error:
                unknown_errors += 1
            elif "registration" not in error["registration"]:
                missing_registration_metadata += 1
            elif "2" not in error["registration"]["registration"]:
                no_stake_key += 1
            else:
                registration_errors[error["registration"]["registration"]["2"]] = error

    # Compare for differences
    compare : dict[str, Any] = {}
    unknown_registrations : list[str] = []
    error_registrations: list[str] = []
    missing_registrations: list[str] = []

    mismatched_voting_power: list[str] = []
    mismatched_delegation: list[str] = []
    mismatched_reward: list[str] = []
    equal_snapshots = 0

    if args.compare is not None:
        raw_compare = json.loads(args.compare.read_text())
        for comp in raw_compare:
            # Index all records being compared.
            stake_pub_key = comp["stake_public_key"]
            compare[stake_pub_key] = comp

            if stake_pub_key in snapshot_index:
                # Check the snapshot is equal
                snapshot_equal = 1

                # Check voting power is the same between records.
                if comp["voting_power"] != snapshot_index[stake_pub_key]["voting_power"]:
                    mismatched_voting_power.append(stake_pub_key)
                    snapshot_equal = 0

                if str(comp["delegations"]) != str(snapshot_index[stake_pub_key]["delegations"]):
                    mismatched_delegation.append(stake_pub_key)
                    snapshot_equal = 0

                if comp["rewards_address"] != snapshot_index[stake_pub_key]["rewards_address"]:
                    mismatched_reward.append(stake_pub_key)
                    snapshot_equal = 0

                equal_snapshots += snapshot_equal

            elif stake_pub_key in registration_errors:
                error_registrations.append(stake_pub_key)
            else:
                unknown_registrations.append(stake_pub_key)

        # Check if valid snapshot has anything NOT in the compared registrations.
        for registration in snapshot_index:
            if registration not in compare:
                missing_registrations.append(registration)

    print("Snapshot Analysis:")
    print(f"  Total Registrations : {len(snapshot)}")
    print(f"  Total CIP-15        : {len(cip_15_snapshot)}")
    print(f"  Total CIP-36 Single : {len(cip_36_single)}")
    print(f"  Total CIP-36 Multi  : {len(cip_36_multi)}")
    print(f"  Total Rejects       : {total_rejects}")

    print()
    print("Stake Address Types:")

    for stake_type, rec in stake_types.items():
        print(f"    {stake_type} : {rec}")

    if len(registration_errors) > 0:
        print()
        print("Registration Errors:")
        print(f"  Total Errors             : {len(registration_errors)}")
        if no_stake_key > 0:
            print(f"  No Stake Key     : {no_stake_key}")
        if unknown_errors > 0:
            print(f"  Unknown Errors   : {unknown_errors}")
        if missing_registration_metadata > 0:
            print(f"  Missing Metadata : {missing_registration_metadata}")

    if len(compare) > 0:
        print()
        print("Snapshot Comparison:")
        print(f"  Total Comparisons       : {len(compare)}")
        print(f"  Equal Snapshots         : {equal_snapshots}")

        print(f"  Mismatched Voting Powers: {len(mismatched_voting_power)}")
        for reg in mismatched_voting_power:
            print(f"        {reg} - snapshot = {snapshot_index[reg]['voting_power']:>15}, compare = {compare[reg]['voting_power']:>15}")

        print(f"  Mismatched Delegations: {len(mismatched_delegation)}")
        for reg in mismatched_delegation:
            print(f"        {reg} - snapshot = {snapshot_index[reg]['delegations']}, compare = {compare[reg]['delegations']}")

        print(f"  Mismatched Rewards Address: {len(mismatched_reward)}")
        for reg in mismatched_delegation:
            print(f"        {reg} - snapshot = {snapshot_index[reg]['rewards_address']}, compare = {compare[reg]['rewards_address']}")


        print(f"  Unknown Registrations : {len(unknown_registrations)}")
        for reg in unknown_registrations:
            print(f"        {reg}")

        print(f"  Error Registrations   : {len(error_registrations)}")
        for reg in error_registrations:
            print(f"        {reg}")

        print(f"  Extra Registrations   : {len(missing_registrations)}")
        for reg in missing_registrations:
            print(f"        {reg}")


def main() -> int:
    """Parse CLI arguments."""
    parser = argparse.ArgumentParser(
        description="Process voting tools snapshot."
    )
    parser.add_argument(
        "--snapshot",
        help="Snapshot file to read.",
        required=True,
        type=is_file,
    )

    parser.add_argument(
        "--errors",
        help="Snapshot errors file to read.",
        required=False,
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
