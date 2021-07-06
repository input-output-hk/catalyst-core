from typing import Dict, Optional, List, Tuple, Generator, TextIO, Union

import sys
import asyncio
import json
import itertools
import enum
import os
from collections import namedtuple

import pydantic
import httpx
import typer

# VIT servicing station models

ADA = "₳"
FUNDED = "FUNDED"
NOT_FUNDED = "NOT_FUNDED"
YES = "YES"
NO = "NO"
NOT_FUNDED_OVER_BUDGET = "Not Funded - Over Budget"
NOT_FUNDED_APPROVAL_THRESHOLD = "Not Funded - Approval Threshold"
LOVELACE_FACTOR = 1000000


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
    decrypted: TallyResult


class PrivateTallyState(pydantic.BaseModel):
    state: DecryptedTally


class PrivateTally(pydantic.BaseModel):
    Private: PrivateTallyState

    @property
    def results(self):
        try:
            return self.Private.state.decrypted.result.results
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


class Challenge(pydantic.BaseModel):
    id: int
    challenge_type: str
    title: str
    description: str
    rewards_total: int
    fund_id: int
    challenge_url: str


# File loaders


def load_json_from_file(file_path: str) -> Dict:
    with open(file_path, encoding="utf-8") as f:
        return json.load(f)


def get_proposals_from_file(proposals_file_path: str) -> Dict[str, Proposal]:
    proposals: Generator[Proposal, None, None] = (
        Proposal(**proposal_data)
        for proposal_data in load_json_from_file(proposals_file_path)
    )
    proposals_dict = {proposal.proposal_id: proposal for proposal in proposals}
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
    proposals = get_proposals_from_file(proposals_file_path)
    voteplan_proposals = get_voteplan_proposals_from_file(voteplan_file_path)
    challeges = get_challenges_from_file(challenges_file_path)
    return proposals, voteplan_proposals, challeges


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
) -> Tuple[Dict[str, Proposal], Dict[str, ProposalStatus]]:
    proposals_task = asyncio.create_task(
        get_proposals_from_api(vit_servicing_station_url)
    )
    voteplans_task = asyncio.create_task(
        get_active_voteplans_from_api(vit_servicing_station_url)
    )
    challenges_task = asyncio.create_task(
        get_challenges_from_api(vit_servicing_station_url)
    )

    proposals = {proposal.proposal_id: proposal for proposal in await proposals_task}
    voteplans_proposals = {
        proposal.proposal_id: proposal
        for proposal in itertools.chain.from_iterable(
            voteplan.proposals for voteplan in await voteplans_task
        )
    }
    challenges = {challenge.id: challenge for challenge in await challenges_task}

    return proposals, voteplans_proposals, challenges


# Checkers


class SanityException(Exception):
    ...


def sanity_check_data(
    proposals: Dict[str, Proposal], voteplan_proposals: Dict[str, ProposalStatus]
):
    if set(proposals.keys()) != set(voteplan_proposals.keys()):
        raise SanityException(
            "Extra proposals found, voteplan proposals do not match servicing station proposals"
        )
    if any(proposal.tally is None for proposal in voteplan_proposals.values()):
        raise SanityException("Some proposal do not have a valid tally available")
    # we checked None before so it is ok to check for next item, we can discard the type checking
    if any(proposal.tally.results is None for proposal in voteplan_proposals.values()):  # type: ignore
        raise SanityException("Some tally results are not available")


# Analyse and compute needed data


def extract_yes_no_votes(proposal: Proposal, voteplan_proposal: ProposalStatus):
    yes_index = proposal.chain_vote_options["yes"]
    no_index = proposal.chain_vote_options["no"]
    # we check before if tally is available, so it should be safe to direct access the data
    yes_result = voteplan_proposal.tally.results[yes_index]  # type: ignore
    no_result = voteplan_proposal.tally.results[no_index]  # type: ignore
    return yes_result, no_result


def calc_approval_threshold(
    proposal: Proposal, voteplan_proposal: ProposalStatus, threshold: float
) -> Tuple[int, bool]:
    yes_result, no_result = extract_yes_no_votes(proposal, voteplan_proposal)
    diff = yes_result - no_result
    success = diff >= (no_result * threshold)
    return diff, success


def calc_vote_difference_and_threshold_success(
    proposals: Dict[str, Proposal],
    voteplan_proposals: Dict[str, ProposalStatus],
    threshold: float,
) -> Dict[str, Tuple[int, bool]]:
    full_ids = set(proposals.keys())
    result = {
        proposal_id: calc_approval_threshold(
            proposals[proposal_id], voteplan_proposals[proposal_id], threshold
        )
        for proposal_id in full_ids
    }
    return result


Result = namedtuple(
    "Result",
    (
        "proposal_id",
        "proposal",
        "overall_score",
        "yes",
        "no",
        "result",
        "meets_approval_threshold",
        "requested_dollars",
        "status",
        "fund_depletion",
        "not_funded_reason",
        "ada_to_be_payed",
        "lovelace_to_be_payed",
        "link_to_ideascale",
    ),
)


def calc_results(
    proposals: Dict[str, Proposal],
    voteplan_proposals: Dict[str, ProposalStatus],
    fund: float,
    conversion_factor: float,
    threshold: float,
) -> List[Result]:
    success_results = calc_vote_difference_and_threshold_success(
        proposals, voteplan_proposals, threshold
    )
    sorted_ids = sorted(
        success_results.keys(), key=lambda x: success_results[x][0], reverse=True
    )
    result_lst = []
    depletion = fund
    for proposal_id in sorted_ids:
        proposal = proposals[proposal_id]
        voteplan_proposal = voteplan_proposals[proposal_id]
        total_result, threshold_success = success_results[proposal_id]
        yes_result, no_result = extract_yes_no_votes(proposal, voteplan_proposal)
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

        ada_to_be_payed = proposal.proposal_funds / conversion_factor if funded else 0

        result = Result(
            proposal_id=proposal_id,
            proposal=proposal.proposal_title,
            overall_score=proposal.proposal_impact_score/100,
            yes=yes_result,
            no=no_result,
            result=total_result,
            meets_approval_threshold=YES if threshold_success else NO,
            requested_dollars=proposal.proposal_funds,
            status=FUNDED if funded else NOT_FUNDED,
            fund_depletion=depletion,
            not_funded_reason=not_funded_reason,
            ada_to_be_payed=ada_to_be_payed,
            lovelace_to_be_payed=ada_to_be_payed * LOVELACE_FACTOR,
            link_to_ideascale=proposal.proposal_url,
        )

        result_lst.append(result)

    return result_lst


def filter_data_by_challenge(
    challenge_id: int,
    proposals: Dict[str, Proposal],
    voteplan_proposals: Dict[str, ProposalStatus],
) -> Tuple[Dict[str, Proposal], Dict[str, ProposalStatus]]:
    proposals = {
        proposal.proposal_id: proposal
        for proposal in proposals.values()
        if proposal.challenge_id == challenge_id
    }
    voteplans = {
        voteplan.proposal_id: voteplan
        for voteplan in voteplan_proposals.values()
        if voteplan.proposal_id in proposals
    }
    return proposals, voteplans


# Output results


def output_csv(results: List[Result]) -> Generator[str, None, None]:
    fields = results[0]._fields
    yield f"{';'.join(fields)}\n"
    yield from (
        f"{';'.join(str(getattr(result, field)) for field in fields)}\n"
        for result in results
    )


def output_json(results: List[Result]) -> Generator[str, None, None]:
    yield json.dumps(list(map(Result._asdict, results)))


def dump_to_file(stream: Generator[str, None, None], out: TextIO):
    out.writelines(stream)


# CLI


def build_path_for_challenge(file_path: str, challenge_name: str) -> str:
    path, suffix = os.path.splitext(file_path)
    return f"{path}_{challenge_name}{suffix}"


class OutputFormat(enum.Enum):
    CSV: str = "csv"
    JSON: str = "json"


def calculate_rewards(
    conversion_factor: float = typer.Option(...),
    output_file: str = typer.Option(...),
    approval_threshold: float = typer.Option(0.15),
    output_format: OutputFormat = typer.Option("csv"),
    proposals_path: Optional[str] = typer.Option(None),
    active_voteplan_path: Optional[str] = typer.Option(None),
    challenges_path: Optional[str] = typer.Option(None),
    vit_station_url: str = typer.Option("https://servicing-station.vit.iohk.io"),
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

    for challenge in challenges.values():
        challenge_proposals, challenge_voteplan_proposals = filter_data_by_challenge(
            challenge.id, proposals, voteplan_proposals
        )
        results = calc_results(
            challenge_proposals,
            challenge_voteplan_proposals,
            challenge.rewards_total,
            conversion_factor,
            approval_threshold,
        )
        out_stream = (
            output_json(results)
            if output_format == OutputFormat.JSON
            else output_csv(results)
        )
        chalenge_ouput_file_path = build_path_for_challenge(
            output_file, challenge.title.replace(" ", "_")
        )
        with open(chalenge_ouput_file_path, "w", encoding="utf-8") as out_file:
            dump_to_file(out_stream, out_file)


if __name__ == "__main__":
    typer.run(calculate_rewards)
