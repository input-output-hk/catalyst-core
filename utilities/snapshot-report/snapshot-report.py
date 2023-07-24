import argparse
from calendar import WEDNESDAY
from dataclasses import dataclass
from datetime import date, datetime, timedelta
import json
from typing import Dict, List


@dataclass
class Voter:
    voting_key: str
    voting_power: int


@dataclass
class Registration:
    delegations: List[str]
    block_time: datetime


class ReportGenerator:
    def __init__(self, registrations: List[Registration], voters: List[Voter]):
        self.total_voters = len(voters)
        self.total_stake = sum([v.voting_power for v in voters])

        delegation_registration_times: Dict[str, datetime] = {}
        for r in registrations:
            for d in r.delegations:
                delegation_registration_times[d] = min(delegation_registration_times.get(d, r.block_time), r.block_time)

        self.new_voters_by_week = {}
        for v in voters:
            bucket = self.date_bucket(delegation_registration_times[v.voting_key])
            self.new_voters_by_week[bucket] = self.new_voters_by_week.get(bucket, 0) + 1

        assert self.total_voters == sum(self.new_voters_by_week.values())

    def pretty_print(self):
        s = f"Total Voters = {self.total_voters}"
        s += f"\nTotal Stake  = {self.total_stake}"

        s += f"\n\nNew Voters (by week):"
        for date in sorted(self.new_voters_by_week.keys()):
            count = self.new_voters_by_week[date]
            s += f"\n{date.isoformat()} = {count}"

        print(s)


    def date_bucket(self, d: datetime) -> date:
        return d.date() + timedelta(days=(WEDNESDAY - d.weekday()) % 7)



def read_catalyst_toolbox_data(path: str) -> List[Voter]:
    voters = []
    voting_keys_with_multiple_contributions = []

    with open(path, "r") as f:
        catalyst_toolbox_data = json.load(f)

        for d in catalyst_toolbox_data:
            contributions = d["contributions"]
            if len(contributions) > 1:
                voting_keys_with_multiple_contributions.append(d)

            hir = d["hir"]

            voters.append(Voter(
                voting_key=fix_voting_key(hir["voting_key"]),
                voting_power=hir["voting_power"],
            ))

    if len(voting_keys_with_multiple_contributions) > 0:
        print("Voting keys with multiple contributions:\n========================================")

        for i in range(len(voting_keys_with_multiple_contributions)):
            d = voting_keys_with_multiple_contributions[i]
            print(f"Voting key = {d['hir']['voting_key']}\nValues = {[c['value'] for c in d['contributions']]}")

            if i < len(voting_keys_with_multiple_contributions) - 1:
                print()
    print()

    return voters


def read_snapshot_tool_data(path: str) -> List[Registration]:
    registrations = []

    with open(path, "r") as f:
        snapshot_tool_data = json.load(f)

        for d in snapshot_tool_data:
            raw_delegations = d["delegations"]

            delegations = []
            if isinstance(raw_delegations, str):
                delegations.append(fix_voting_key(raw_delegations))
            elif isinstance(raw_delegations, list):
                delegations = [fix_voting_key(x[0]) for x in raw_delegations]
            else:
                raise Exception("Invalid delegations format (neither CIP-15 or CIP-36)")

            registrations.append(Registration(
                delegations=delegations,
                block_time=datetime.fromisoformat(d["block_time"]),
            ))

    return registrations


def fix_voting_key(vk: str) -> str:
    s = vk
    if vk.startswith("0x"):
        s = vk[2:]

    return "0x" + s.zfill(64)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--catalyst-toolbox-file", help="Path to the catalyst-toolbox output file", required=True)
    parser.add_argument("--snapshot-tool-file", help="Path to the snapshot_tool output file", required=True)
    args = parser.parse_args()

    registrations = read_snapshot_tool_data(args.snapshot_tool_file)
    voters = read_catalyst_toolbox_data(args.catalyst_toolbox_file)

    ReportGenerator(registrations=registrations, voters=voters).pretty_print()
