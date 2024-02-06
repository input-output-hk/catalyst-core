# coding: utf-8
from typing import Dict, Optional, List, Tuple, Generator, TextIO, Union, Any, Set

import sys
import asyncio
import json
import csv
import itertools
import enum
import os
import re
from collections import namedtuple
from io import StringIO

import pydantic
import httpx
import typer
import yaml

# VIT servicing station models

ADA = "₳"
FUNDED = "FUNDED"
NOT_FUNDED = "NOT_FUNDED"
YES = "YES"
NO = "NO"
NOT_FUNDED_OVER_BUDGET = "Not Funded - Over Budget"
NOT_FUNDED_APPROVAL_THRESHOLD = "Not Funded - Approval Threshold"
LOVELACE_FACTOR = 1000000

class Challenge(pydantic.BaseModel):
    id: int
    challenge_type: str
    title: str
    description: str
    rewards_total: int
    fund_id: int
    challenge_url: str

class Proposal(pydantic.BaseModel):
    internal_id: int
    proposal_id: str
    proposal_title: str
    proposal_funds: int
    proposal_url: str
    proposal_impact_score: int
    chain_proposal_id: str
    chain_proposal_index: int
    chain_vote_options: Dict[str, int]
    fund_id: int
    challenge_id: int
    challenge_type: str
    challenge: Challenge
    
    @pydantic.computed_field
    @property
    def ideascale_url(self) -> str:
        return f"https://cardano.ideascale.com/c/idea/{self.proposal_id}"

# Jormungandr models


class Options(pydantic.BaseModel):
    start: int
    end: int


class Results(pydantic.BaseModel):
    results: List[int]


# we mimic the tally
class TallyResult(pydantic.BaseModel):
    result: Results


class DecryptedTally(pydantic.BaseModel):
    Decrypted: TallyResult


class PrivateTallyState(pydantic.BaseModel):
    state: DecryptedTally


class PrivateTally(pydantic.BaseModel):
    Private: PrivateTallyState

    @property
    def results(self):
        try:
            return self.Private.state.Decrypted.result.results
        except AttributeError:
            return None


class PublicTally(pydantic.BaseModel):
    Public: TallyResult

    @property
    def results(self):
        try:
            return self.Public.result.results
        except AttributeError:
            return None


class ProposalStatus(pydantic.BaseModel):
    index: int
    proposal_id: str
    options: Options
    tally: Optional[Union[PublicTally, PrivateTally]]
    votes_cast: int


class VoteplanStatus(pydantic.BaseModel):
    id: str
    payload: str
    proposals: List[ProposalStatus]


class Result(pydantic.BaseModel):
    internal_id: int
    proposal_id: str
    proposal: str
    overall_score: float
    yes: int
    abstain: int
    meets_threshold: str
    requested_funds: int
    status: str
    fund_depletion: int
    not_funded_reason: str
    website_url: str
    ideascale_url: str
    challenge_title: str
    challenge_id: int
    votes_cast: int

class Winner(pydantic.BaseModel):
    internal_id: int
    proposal_id: str
    project_id: int
    proposal_title: str
    requested_funds: int
    website_url: str
    ideascale_url: str
    challenge_title: str
    challenge_id: int
    milestone_qty: int

# File loaders


def load_json_from_file(file_path: str) -> Dict:
    with open(file_path, encoding="utf-8") as f:
        return json.load(f)


def get_proposals_from_file(proposals_file_path: str, challenges: Dict[int, Challenge]) -> Dict[str, Proposal]:
    proposals: Generator[Proposal, None, None] = (
        Proposal(**proposal_data, challenge=challenges[proposal_data['challenge_id']])
        for proposal_data in load_json_from_file(proposals_file_path)
    )
    proposals_dict = {proposal.chain_proposal_id: proposal for proposal in proposals}
    return proposals_dict


def get_voteplan_proposals_from_file(
    voteplan_file_path: str,
) -> Dict[str, ProposalStatus]:
    voteplans_status: Generator[VoteplanStatus, None, None] = (
        VoteplanStatus(**voteplan)
        for voteplan in load_json_from_file(voteplan_file_path)
    )
    flattened_voteplan_proposals = itertools.chain.from_iterable(
        voteplan_status.proposals for voteplan_status in voteplans_status
    )

    voteplan_proposals_dict = {
        proposal.proposal_id: proposal for proposal in flattened_voteplan_proposals
    }
    return voteplan_proposals_dict


def get_challenges_from_file(challenges_file_path: str) -> Dict[int, Challenge]:
    challenges: Generator[Challenge, None, None] = (
        Challenge(**challenge)
        for challenge in load_json_from_file(challenges_file_path)
    )
    challenges_dict = {challenge.id: challenge for challenge in challenges}
    return challenges_dict


def get_proposals_voteplans_and_challenges_from_files(
    proposals_file_path: str, voteplan_file_path: str, challenges_file_path: str
) -> Tuple[Dict[str, Proposal], Dict[str, ProposalStatus], Dict[int, Challenge]]:
    voteplan_proposals = get_voteplan_proposals_from_file(voteplan_file_path)
    challenges = get_challenges_from_file(challenges_file_path)
    proposals = get_proposals_from_file(proposals_file_path, challenges)
    return proposals, voteplan_proposals, challenges


def get_excluded_proposals_from_file(excluded_proposals_path: str) -> List[str]:
    with open(excluded_proposals_path, encoding="utf-8") as f:
        return json.load(f)


# API loaders


async def get_data_from_api(base_url: str, endpoint: str, cls: "T") -> List["T"]:
    async with httpx.AsyncClient() as client:
        result = await client.get(f"{base_url}{endpoint}")
        assert result.status_code == 200
        return [cls(**data) for data in result.json()]


async def get_proposals_from_api(vit_servicing_station_url: str) -> List[Proposal]:
    return await get_data_from_api(
        vit_servicing_station_url, "/api/v0/proposals", Proposal
    )


async def get_active_voteplans_from_api(
    vit_servicing_station_url: str,
) -> List[VoteplanStatus]:
    return await get_data_from_api(
        vit_servicing_station_url, "/api/v0/vote/active/plans", VoteplanStatus
    )


async def get_challenges_from_api(vit_servicing_station_url: str) -> List[Challenge]:
    return await get_data_from_api(
        vit_servicing_station_url, "/api/v0/challenges", Challenge
    )


async def get_proposals_voteplans_and_challenges_from_api(
    vit_servicing_station_url: str,
) -> Tuple[Dict[str, Proposal], Dict[str, ProposalStatus], Dict[str, Challenge]]:
    proposals_task = asyncio.create_task(
        get_proposals_from_api(vit_servicing_station_url)
    )
    voteplans_task = asyncio.create_task(
        get_active_voteplans_from_api(vit_servicing_station_url)
    )
    challenges_task = asyncio.create_task(
        get_challenges_from_api(vit_servicing_station_url)
    )

    proposals = {
        proposal.chain_proposal_id: proposal for proposal in await proposals_task
    }
    voteplans_proposals = {
        proposal.chain_proposal_id: proposal
        for proposal in itertools.chain.from_iterable(
            voteplan.proposals for voteplan in await voteplans_task
        )
    }
    challenges = {challenge.id: challenge for challenge in await challenges_task}

    return proposals, voteplans_proposals, challenges


def load_block0_data(block0_path: str) -> Dict[str, Any]:
    with open(block0_path, encoding="utf8") as f:
        return yaml.load(f, Loader=yaml.FullLoader)


# Checkers


class SanityException(Exception):
    ...


def sanity_check_data(
    proposals: Dict[str, Proposal], voteplan_proposals: Dict[str, ProposalStatus]
):
    proposals_set = set(proposals.keys())
    voteplan_proposals_set = set(voteplan_proposals.keys())
    if proposals_set != voteplan_proposals_set:
        from pprint import pformat

        diff = proposals_set.difference(voteplan_proposals_set)
        raise SanityException(
            f"Extra proposals found, voteplan proposals do not match servicing station proposals: \n{pformat(diff)}"
        )
    if any(proposal.tally is None for proposal in voteplan_proposals.values()):
        raise SanityException("Some proposal do not have a valid tally available")
    # we checked None before so it is ok to check for next item, we can discard the type checking
    if any(proposal.tally.results is None for proposal in voteplan_proposals.values()):  # type: ignore
        raise SanityException("Some tally results are not available")


# Analyse and compute needed data


def extract_yes_abstain_votes(proposal: Proposal, voteplan_proposal: ProposalStatus):
    yes_index = int(proposal.chain_vote_options["yes"])
    abstain_index = int(proposal.chain_vote_options["no"])
    # we check before if tally is available, so it should be safe to direct access the data
    yes_result = int(voteplan_proposal.tally.results[yes_index])
    abstain_result = int(voteplan_proposal.tally.results[abstain_index])
    return yes_result, abstain_result


def calc_approval_threshold(
    proposal: Proposal,
    voteplan_proposal: ProposalStatus,
    total_stake_threshold: float,
) -> Tuple[int, bool]:
    yes_result, abstain_result = extract_yes_abstain_votes(proposal, voteplan_proposal)
    pass_total_threshold = yes_result >= float(total_stake_threshold)
    return yes_result, pass_total_threshold


def calc_vote_difference_and_threshold_success(
    proposals: Dict[str, Proposal],
    voteplan_proposals: Dict[str, ProposalStatus],
    total_stake_threshold: float,
) -> Dict[str, Tuple[int, bool]]:
    full_ids = set(proposals.keys())
    result = {
        proposal_id: calc_approval_threshold(
            proposals[proposal_id],
            voteplan_proposals[proposal_id],
            total_stake_threshold,
        )
        for proposal_id in full_ids
    }
    return result





def calc_results(
    proposals: Dict[str, Proposal],
    voteplan_proposals: Dict[str, ProposalStatus],
    funds: float,
    total_stake_threshold: float,
) -> List[Result]:
    success_results = calc_vote_difference_and_threshold_success(
        proposals, voteplan_proposals, total_stake_threshold
    )
    sorted_ids = sorted(
        success_results.keys(), key=lambda x: success_results[x][0], reverse=True
    )
    result_lst = []
    depletion = funds
    for proposal_id in sorted_ids:
        proposal = proposals[proposal_id]
        voteplan_proposal = voteplan_proposals[proposal_id]
        total_result, threshold_success = success_results[proposal_id]
        yes_result, abstain_result = extract_yes_abstain_votes(proposal, voteplan_proposal)
        funded = all(
            (threshold_success, depletion > 0, depletion >= proposal.proposal_funds)
        )
        not_funded_reason = (
            ""
            if funded
            else (
                NOT_FUNDED_APPROVAL_THRESHOLD
                if not threshold_success
                else NOT_FUNDED_OVER_BUDGET
            )
        )

        if funded:
            depletion -= proposal.proposal_funds

        result = Result(
            internal_id=proposal.proposal_id,
            proposal_id=proposal_id,
            proposal=proposal.proposal_title,
            overall_score=proposal.proposal_impact_score / 100,
            yes=yes_result,
            abstain=abstain_result,
            meets_threshold=YES if threshold_success else NO,
            requested_funds=proposal.proposal_funds,
            status=FUNDED if funded else NOT_FUNDED,
            fund_depletion=depletion,
            not_funded_reason=not_funded_reason,
            website_url=proposal.proposal_url,
            ideascale_url=proposal.ideascale_url,
            challenge_id=proposal.challenge.id,
            challenge_title=proposal.challenge.title,
            votes_cast=voteplan_proposal.votes_cast
        )

        result_lst.append(result)

    return result_lst, depletion


def filter_data_by_challenge(
    challenge_id: int,
    proposals: Dict[str, Proposal],
    voteplan_proposals: Dict[str, ProposalStatus],
) -> Tuple[Dict[str, Proposal], Dict[str, ProposalStatus]]:
    proposals = {
        proposal.chain_proposal_id: proposal
        for proposal in proposals.values()
        if proposal.challenge_id == challenge_id
    }
    voteplans = {
        voteplan.proposal_id: voteplan
        for voteplan in voteplan_proposals.values()
        if voteplan.proposal_id in proposals
    }
    return proposals, voteplans


def filter_excluded_proposals(
    proposals: Dict[str, Proposal], excluded: Set[str]
) -> Dict[str, Proposal]:
    return {
        k: v
        for k, v in proposals.items()
        if all(_id not in excluded for _id in (v.proposal_id, v.chain_proposal_id))
    }


def calculate_total_stake_from_block0_configuration(
    block0_config: Dict[str, Dict], committee_keys: List[str]
):
    funds = (
        initial["fund"] for initial in block0_config["initial"] if "fund" in initial
    )
    return sum(
        fund["value"]
        for fund in itertools.chain.from_iterable(funds)
        if fund["address"] not in [key for key in committee_keys]
    )

def calc_leftovers(results, remaining_funds):
    leftovers_candidates = sorted([
        result
        for result in results
        if result.status == NOT_FUNDED and result.meets_threshold == YES
    ], key=lambda x: x.yes, reverse=True)

    depletion = remaining_funds
    for candidate in leftovers_candidates:
        funded = depletion >= candidate.requested_funds
        not_funded_reason = (
            ""
            if funded
            else NOT_FUNDED_OVER_BUDGET
        )
        if funded:
            depletion -= candidate.requested_funds
        candidate.status = FUNDED if funded else NOT_FUNDED
        candidate.fund_depletion = depletion
        candidate.not_funded_reason = not_funded_reason
    
    return leftovers_candidates

def pick_milestones_qty(winner, limits, qty):
    idx = next((i for i, l in enumerate(limits) if winner.requested_funds > l), None)
    return qty[idx]

def generate_winners(results, fund_prefix, milestones_limit, milestones_qty):
    winners = []
    _winners = sorted([r for r in results if r.status == FUNDED], key=lambda r: r.proposal)
    for idx, _winner in enumerate(_winners):
        winners.append(Winner(
            **_winner.dict(),
            proposal_title=_winner.proposal,
            project_id=fund_prefix + idx,
            milestone_qty=pick_milestones_qty(_winner, milestones_limit, milestones_qty)
        ))
    return winners

# Output results


def output_csv(results: List[Result], f: TextIO):
    elements = [r.dict() for r in results]
    fields = elements[0].keys()
    writer = csv.DictWriter(f, fields)
    writer.writeheader()
    writer.writerows(elements)


def output_json(results: List[Result], f: TextIO):
    json.dump(list(map(Result._asdict, results)), f)


# CLI

class OutputFormat(enum.Enum):
    CSV: str = "csv"
    JSON: str = "json"


def build_path_for_challenge(file_path: str, challenge_name: str, output_format: OutputFormat) -> str:
    path, suffix = os.path.splitext(file_path)
    suffix = 'json' if (output_format == OutputFormat.JSON) else 'csv'
    return f"{path}_{challenge_name}.{suffix}"


def save_results(output_path: str, title: str, output_format: OutputFormat, results: List[Results]):
    challenge_output_file_path = build_path_for_challenge(
        output_path,
        re.sub(
            r"(?u)[^-\w.]", "", title.replace(" ", "_").replace(":", "_")
        ),
        output_format
    )

    with open(
        challenge_output_file_path, "w", encoding="utf-8", newline=""
    ) as out_file:
        if output_format == OutputFormat.JSON:
            output_json(results, out_file)
        elif output_format == OutputFormat.CSV:
            output_csv(results, out_file)


def calculate_rewards(
    output_file: str = typer.Option(...),
    block0_path: str = typer.Option(...),
    total_stake_threshold: float = typer.Option(0.01),
    output_format: OutputFormat = typer.Option("csv"),
    proposals_path: Optional[str] = typer.Option(None),
    excluded_proposals_path: Optional[str] = typer.Option(None),
    active_voteplan_path: Optional[str] = typer.Option(None),
    challenges_path: Optional[str] = typer.Option(None),
    vit_station_url: str = typer.Option("https://servicing-station.vit.iohk.io"),
    committee_keys_path: Optional[str] = typer.Option(None),
    fund_prefix: int = typer.Option(1100001),
    milestones_limit: List[int] = typer.Option([0, 75000, 150000, 300000]),
    milestones_qty: List[int] = typer.Option([3, 4, 5, 6])
):
    """
    Calculate catalyst rewards after tallying process.
    If all --proposals-path, --active-voteplan-path and --challenges_path are provided.
    Then, data is loaded from the json files on those locations.
    Otherwise data is requested to the proper API endpoints pointed to the --vit-station-url option.
    Rewards are written into a separated file for each challenge. File is constructed via the --output-file.
    For example /out/rewards.csv with challenges [challenge_1, challenge_2] will generate /out/rewards_challenge_1.csv
    and /out/rewards_challenge_2.csv files.
    """
    if all(
        path is not None
        for path in (proposals_path, active_voteplan_path, challenges_path)
    ):
        proposals, voteplan_proposals, challenges = (
            # we already check that both paths are not None, we can disable type checking here
            get_proposals_voteplans_and_challenges_from_files(
                proposals_path, active_voteplan_path, challenges_path
            )  # type: ignore
        )
    else:
        proposals, voteplan_proposals, challenges = asyncio.run(
            get_proposals_voteplans_and_challenges_from_api(vit_station_url)
        )

    try:
        sanity_check_data(proposals, voteplan_proposals)
    except SanityException as e:
        print(f"{e}")
        sys.exit(1)

    excluded_proposals: Set[str] = (
        set(get_excluded_proposals_from_file(excluded_proposals_path))
        if excluded_proposals_path
        else set()
    )

    proposals = filter_excluded_proposals(proposals, excluded_proposals)

    block0_config = load_block0_data(block0_path)
    committee_keys = (
        load_json_from_file(committee_keys_path) if committee_keys_path else []
    )
    total_stake = calculate_total_stake_from_block0_configuration(
        block0_config, committee_keys
    )
    # minimum amount of stake needed for a proposal to be accepted
    total_stake_approval_threshold = float(total_stake_threshold) * float(total_stake)

    total_remaining_funds = 0

    all_results = []

    for challenge in challenges.values():
        challenge_proposals, challenge_voteplan_proposals = filter_data_by_challenge(
            challenge.id, proposals, voteplan_proposals
        )
        results, remaining_funds = calc_results(
            challenge_proposals,
            challenge_voteplan_proposals,
            challenge.rewards_total,
            total_stake_approval_threshold,
        )

        total_remaining_funds += remaining_funds
        all_results += results

        save_results(output_file, challenge.title, output_format, results)
        
    print(f"Total Stake: {total_stake}")
    print(f"Total Stake threshold: {total_stake_approval_threshold}")
    print(f"Leftover budget: {total_remaining_funds}")
    
    leftover_results = calc_leftovers(all_results, total_remaining_funds)
    save_results(output_file, 'leftovers', output_format, leftover_results)

    milestones_limit.reverse()
    milestones_qty.reverse()
    winners = generate_winners(all_results + leftover_results, fund_prefix, milestones_limit, milestones_qty)
    save_results(output_file, 'winners', output_format, winners)

if __name__ == "__main__":
    typer.run(calculate_rewards)
