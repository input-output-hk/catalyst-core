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


def compare_reg_error(reg:dict, error:dict) -> bool:
    """Compare a registration, with an error record."""
    try:
        delegations_match = str(reg["delegations"]) == str(error["registration"]["61284"]["1"])
        rewards_match = reg["rewards_address"] == error["registration"]["61284"]["3"]
        return delegations_match and rewards_match
    except:
        return False

def analyze_snapshot(args: argparse.Namespace):
    """Convert a snapshot into a format supported by SVE1."""

    # make errors output filename
    snapshot_errors_fname = args.snapshot.with_suffix(".errors" + args.snapshot.suffix)
    snapshot_unregistered_fname = args.snapshot.with_suffix(".unregistered" + args.snapshot.suffix)

    # Load json file
    snapshot = json.loads(args.snapshot.read_text())
    snapshot_index: dict[str, Any] = {}

    snapshot_errors = json.loads(snapshot_errors_fname.read_text())
    snapshot_unregistered = json.loads(snapshot_unregistered_fname.read_text())

    cip_15_snapshot: list[dict[str, Any]] = []
    cip_36_single: list[dict[str, Any]] = []
    cip_36_multi: list[dict[str, Any]] = []

    total_rejects = 0
    total_registered_value = 0

    for registration in snapshot:
        # Index the registrations
        stake_pub_key = registration["stake_public_key"]
        snapshot_index[stake_pub_key] = registration

        total_registered_value += registration["voting_power"]

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

    # Index Errors
    registration_obsolete: dict[str, Any] = {}
    decode_errors: list[Any] = []
    other_errors: dict[str, Any] = {}

    # Index the registration errors by their stake key.
    for error in snapshot_errors:
        errors = error["errors"]
        if errors == ["ObsoleteRegistration"]:
            stake = error["registration"]["61284"]["2"]
            if stake in registration_obsolete:
                registration_obsolete[stake].append(error)
            else:
                registration_obsolete[stake] = [error]
        else:
            try:
                stake = error["registration"]["61284"]["2"]
                if stake in other_errors:
                    other_errors[stake].append(error)
                else:
                    other_errors[stake] = [error]
            except:
                decode_errors.append(error)

    # Compare for differences
    compare: dict[str, Any] = {}
    unknown_registrations: list[str] = []
    missing_registrations: list[str] = []

    mismatched: dict[str, Any] = {}
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
                if (comp["voting_power"] != snapshot_index[stake_pub_key]["voting_power"]) or  (
                    str(comp["delegations"]) != str(snapshot_index[stake_pub_key]["delegations"])) or (
                    comp["rewards_address"] != snapshot_index[stake_pub_key]["rewards_address"]):
                    mismatched[stake_pub_key] = comp
                    snapshot_equal = 0

                equal_snapshots += snapshot_equal

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

    #if len(registration_errors) > 0:
    #    print()
    #    print("Registration Errors:")
    #    print(f"  Total Errors             : {len(registration_errors)}")
    #    if no_stake_key > 0:
    #        print(f"  No Stake Key     : {no_stake_key}")
    #    if unknown_errors > 0:
    #        print(f"  Unknown Errors   : {unknown_errors}")
    #    if missing_registration_metadata > 0:
    #        print(f"  Missing Metadata : {missing_registration_metadata}")

    if len(compare) > 0:
        print()
        print("Snapshot Comparison:")
        print(f"  Total Comparisons       : {len(compare)}")
        print(f"  Equal Snapshots         : {equal_snapshots}")

        for mismatched_stake, mismatched_data in mismatched.items():
            if mismatched_stake in registration_obsolete:
                for obs in registration_obsolete[mismatched_stake]:
                    if compare_reg_error(mismatched_data, obs):
                        reg_nonce = snapshot_index[mismatched_stake]['nonce']
                        reg_txid = snapshot_index[mismatched_stake]['tx_id']
                        obs_nonce = obs['registration']['61284']['4']
                        obs_txid  = obs['registration']['tx_id']
                        print(f"        {mismatched_stake} was obsoleted (nonce:tx_id) .  Reg = {reg_nonce}:{reg_txid}, Compare = {obs_nonce}:{obs_txid}")
            else:
                print(f"        {mismatched_stake} was different.")



        print(f"  Unknown Registrations : {len(unknown_registrations)}")
        for reg in unknown_registrations:
            print(f"        {reg}")

        print(f"  Extra Registrations   : {len(missing_registrations)}")
        for reg in missing_registrations:
            print(f"        {reg}")

        total_unregistered = len(snapshot_unregistered)
        value_unregistered = 0
        for value in snapshot_unregistered.values():
            value_unregistered += value

        print(f"  Total Registrations = Total Voting Power  : {len(snapshot):>10}  = {total_registered_value/1000000:>25} ADA")
        print(f"  Total Unregistered  = Total Voting Power  : {total_unregistered:>10}  = {value_unregistered/1000000:>25} ADA")

        staked_total = len(snapshot) + total_unregistered
        staked_total_value = total_registered_value + value_unregistered

        reg_pct = 100.0 / staked_total * len(snapshot)
        val_pct = 100.0 / staked_total_value * total_registered_value

        print(f"  Registered%         = VotingPower %       : {reg_pct:>10.4}% = {val_pct:>23.4}%")



def main() -> int:
    """Parse CLI arguments."""
    parser = argparse.ArgumentParser(
        description="Process voting tools snapshot."
    )
    parser.add_argument(
        "--snapshot",
        help="Rust Snapshot file to read.",
        required=True,
        type=is_file,
    )

    parser.add_argument(
        "--compare",
        help="Haskell Snapshot file to compare with.",
        required=False,
        type=is_file,
    )

    args = parser.parse_args()
    analyze_snapshot(args)
    return 0


if __name__ == "__main__":
    sys.exit(main())
