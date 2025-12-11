# coding: utf-8
from typing import (
    Dict,
    Optional,
    List,
    Tuple,
    Generator,
    TextIO,
    Union,
    Any,
    Set,
    Mapping,
)

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
import asyncio
import aiohttp
from rich import print
from asyncio import run as aiorun
from copy import deepcopy
from fractions import Fraction


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


class Author(pydantic.BaseModel):
    """Represents an author."""

    id: int
    name: str
    email: str
    user_name: str = pydantic.Field(alias="userName")


# Ideascale models


class IdeascaleProposal(pydantic.BaseModel):
    id: int
    title: str
    authors: List[Author] = pydantic.Field(default=[])

    @pydantic.model_validator(mode="before")
    @classmethod
    def assign_authors_if_any(cls, values):
        """Assign proposers/co-proposers merging different ideascale fields."""
        authors = []
        if "authorInfo" in values:
            authors.append(Author(**values["authorInfo"]))
        if "contributors" in values:
            for contributor in values["contributors"]:
                authors.append(Author(**contributor))
        values["authors"] = authors
        return values


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
    yes: int
    abstain: Optional[int] = None
    no: Optional[int] = None
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
    vote_result: Optional[int] = None


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
    authors: List[Author] = pydantic.Field([])

    def dict(self, **kwargs):
        # Override std dict to list all authors in different columns
        output = super().dict(**kwargs)
        _output = {}
        for k, v in output.items():
            if k == "authors":
                for idx, author in enumerate(v):
                    _output[f"{k}_{idx}"] = author["email"]
            else:
                _output[k] = v
        return _output


# Ideascale interface


class JsonHttpClient:
    """HTTP Client for JSON APIs."""

    def __init__(self, api_url: str):
        """Initialize a new instance of JsonHttpClient."""
        self.api_url = api_url
        self.request_counter = 0

    async def get(self, path: str, headers: Mapping[str, str] = {}):
        """Execute a GET request against a service."""
        url = f"{self.api_url}{path}"

        async with aiohttp.ClientSession() as session:
            async with session.get(url, headers=headers) as r:
                content = b""

                async for c, _ in r.content.iter_chunks():
                    content += c

                if r.status == 200:
                    parsed_json = json.loads(content)
                    return parsed_json
                else:
                    raise GetFailed(r.status, r.reason, content)


class GetFailed(Exception):
    """Raised when a request fails."""

    def __init__(self, status, reason, content):
        """Initialize a new instance of GetFailed."""
        super().__init__(f"{status} {reason}\n{content})")


class IdeascaleImporter:
    """Interface with IdeaScale API."""

    def __init__(
        self, api_key: str, api_url: str = "https://temp-cardano-sandbox.ideascale.com"
    ):
        """Initialize entities."""
        self.api_key = api_key
        self.api_url = api_url
        self.inner = JsonHttpClient(self.api_url)
        self.N_WORKERS = 3

        self.proposals: List[IdeascaleProposal] = []

    async def import_proposals(self, stage_ids: List[int], page_size: int = 50):
        """Get all ideas from the stage with the given id.

        Pages are requested concurrently until the latest one fails
        which signals that that are no more pages left.
        """

        class WorkerData:
            def __init__(self, stage_id):
                self.stage_id = stage_id

                self.page: int = 0
                self.done: bool = False
                self.proposals: List[IdeascaleProposal] = []

        async def worker(d: WorkerData, stage_id: int):
            while True:
                if d.done:
                    break

                p = d.page
                d.page += 1

                res = await self._get(
                    f"/a/rest/v1/stages/{stage_id}/ideas/{p}/{page_size}"
                )

                res_proposals: List[IdeascaleProposal] = []
                for i in res:
                    if i["stageId"] == stage_id:
                        res_proposals.append(IdeascaleProposal(**i))

                d.proposals.extend(res_proposals)

                if len(res_proposals) < page_size:
                    d.done = True

        d = {}
        for stage_id in stage_ids:
            print(f"Start proposal requests for stage: {stage_id}")
            d = WorkerData(stage_id)
            worker_tasks = [
                asyncio.create_task(worker(d, stage_id)) for _ in range(self.N_WORKERS)
            ]
            for task in worker_tasks:
                await task
            self.proposals.extend(d.proposals)

    async def _get(self, path: str):
        """Execute a GET request."""
        headers = {"api_token": self.api_key}
        return await self.inner.get(path, headers)


# File loaders


def load_json_from_file(file_path: str) -> Dict:
    with open(file_path, encoding="utf-8") as f:
        return json.load(f)


def get_proposals_from_file(
    proposals_file_path: str, challenges: Dict[int, Challenge]
) -> Dict[str, Proposal]:
    proposals: Generator[Proposal, None, None] = (
        Proposal(**proposal_data, challenge=challenges[proposal_data["challenge_id"]])
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


class SanityException(Exception): ...


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


class WinnerSelectionRule(enum.Enum):
    YES_ONLY: str = "yes_only"
    YES_NO_DIFF: str = "yes_no_diff"


def extract_choices_votes(proposal: Proposal, voteplan_proposal: ProposalStatus):
    yes_index = int(proposal.chain_vote_options["yes"])
    no_index = int(proposal.chain_vote_options["no"])
    # we check before if tally is available, so it should be safe to direct access the data
    yes_result = int(voteplan_proposal.tally.results[yes_index])
    no_result = int(voteplan_proposal.tally.results[no_index])
    return yes_result, no_result


def calc_approval_threshold(
    proposal: Proposal,
    voteplan_proposal: ProposalStatus,
    total_stake_threshold: float,
    winner_selection_rule: WinnerSelectionRule,
    relative_threshold: float,
) -> Tuple[int, bool]:
    yes_result, second_choice_result = extract_choices_votes(
        proposal, voteplan_proposal
    )
    pass_relative_threshold = (
        (yes_result - second_choice_result) / (yes_result + second_choice_result)
    ) >= float(relative_threshold)
    if winner_selection_rule == WinnerSelectionRule.YES_ONLY:
        vote_result = yes_result
        pass_total_threshold = yes_result >= float(total_stake_threshold)
    elif winner_selection_rule == WinnerSelectionRule.YES_NO_DIFF:
        vote_result = yes_result - second_choice_result
        pass_total_threshold = (yes_result + second_choice_result) >= float(
            total_stake_threshold
        )
    threshold_rules = pass_total_threshold and pass_relative_threshold
    return vote_result, threshold_rules


def calc_vote_value_and_threshold_success(
    proposals: Dict[str, Proposal],
    voteplan_proposals: Dict[str, ProposalStatus],
    total_stake_threshold: float,
    winner_selection_rule: WinnerSelectionRule,
    relative_threshold: float,
) -> Dict[str, Tuple[int, bool]]:
    full_ids = set(proposals.keys())
    result = {
        proposal_id: calc_approval_threshold(
            proposals[proposal_id],
            voteplan_proposals[proposal_id],
            total_stake_threshold,
            winner_selection_rule,
            relative_threshold,
        )
        for proposal_id in full_ids
    }
    return result


def calc_results(
    proposals: Dict[str, Proposal],
    voteplan_proposals: Dict[str, ProposalStatus],
    funds: float,
    total_stake_threshold: float,
    winner_selection_rule: WinnerSelectionRule,
    relative_threshold: float,
) -> List[Result]:
    success_results = calc_vote_value_and_threshold_success(
        proposals,
        voteplan_proposals,
        total_stake_threshold,
        winner_selection_rule,
        relative_threshold,
    )
    sorted_ids = sorted(
        success_results.keys(), key=lambda x: success_results[x][0], reverse=True
    )
    result_lst = []
    depletion = funds
    for proposal_id in sorted_ids:
        proposal = proposals[proposal_id]
        voteplan_proposal = voteplan_proposals[proposal_id]
        vote_result, threshold_success = success_results[proposal_id]
        yes_result, second_choice_result = extract_choices_votes(
            proposal, voteplan_proposal
        )
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
            meets_threshold=YES if threshold_success else NO,
            requested_funds=proposal.proposal_funds,
            status=FUNDED if funded else NOT_FUNDED,
            fund_depletion=depletion,
            not_funded_reason=not_funded_reason,
            website_url=proposal.proposal_url,
            ideascale_url=proposal.ideascale_url,
            challenge_id=proposal.challenge.id,
            challenge_title=proposal.challenge.title,
            votes_cast=voteplan_proposal.votes_cast,
        )

        if winner_selection_rule == WinnerSelectionRule.YES_ONLY:
            result.abstain = second_choice_result
        if winner_selection_rule == WinnerSelectionRule.YES_NO_DIFF:
            result.vote_result = vote_result
            result.no = second_choice_result

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
    block0_config: Dict[str, Dict], committee_keys: List[str], gamma: Fraction
):
    funds = (
        initial["fund"] for initial in block0_config["initial"] if "fund" in initial
    )
    return sum(
        fund["value"] ** gamma
        for fund in itertools.chain.from_iterable(funds)
        if fund["address"] not in [key for key in committee_keys]
    )


def extract_relevant_choice(x, winner_selection_rule):
    if winner_selection_rule == WinnerSelectionRule.YES_ONLY:
        return x.yes
    elif winner_selection_rule == WinnerSelectionRule.YES_NO_DIFF:
        return x.vote_result


def calc_leftovers(
    results, remaining_funds, excluded_categories, winner_selection_rule
):
    leftovers_candidates = sorted(
        [
            result
            for result in deepcopy(results)
            if (
                result.status == NOT_FUNDED
                and result.meets_threshold == YES
                and result.challenge_id not in excluded_categories
            )
        ],
        key=lambda x: extract_relevant_choice(x, winner_selection_rule),
        reverse=True,
    )

    depletion = remaining_funds
    for candidate in leftovers_candidates:
        funded = depletion >= candidate.requested_funds
        not_funded_reason = "" if funded else NOT_FUNDED_OVER_BUDGET
        if funded:
            depletion -= candidate.requested_funds
        candidate.status = FUNDED if funded else NOT_FUNDED
        candidate.fund_depletion = depletion
        candidate.not_funded_reason = not_funded_reason

    return leftovers_candidates, depletion


def pick_milestones_qty(winner, limits, qty):
    idx = next((i for i, l in enumerate(limits) if winner.requested_funds > l), None)
    return qty[idx]


def generate_winners(
    results, fund_prefix, milestones_limit, milestones_qty, _ideascale_proposals
):
    ideascale_proposals = {p.id: p for p in _ideascale_proposals}
    winners = []
    _winners = sorted(
        [r for r in results if r.status == FUNDED], key=lambda r: r.proposal.lower()
    )
    for idx, _winner in enumerate(_winners):
        winner = Winner(
            **_winner.dict(),
            proposal_title=_winner.proposal,
            project_id=fund_prefix + idx,
            milestone_qty=pick_milestones_qty(
                _winner, milestones_limit, milestones_qty
            ),
        )
        if winner.internal_id in ideascale_proposals.keys():
            winner.authors = ideascale_proposals[winner.internal_id].authors
        winners.append(winner)
    return winners


# Output results


def output_csv(results: List[Result], f: TextIO):
    elements = [r.dict(exclude_none=True) for r in results]
    keys = max([e.keys() for e in elements], key=len)
    fields = keys
    writer = csv.DictWriter(f, fields)
    writer.writeheader()
    writer.writerows(elements)


def output_json(results: List[Result], f: TextIO):
    json.dump(list(map(Result._asdict, results)), f)


# CLI


class OutputFormat(enum.Enum):
    CSV: str = "csv"
    JSON: str = "json"


def build_path_for_challenge(
    file_path: str, challenge_name: str, output_format: OutputFormat
) -> str:
    path, suffix = os.path.splitext(file_path)
    suffix = "json" if (output_format == OutputFormat.JSON) else "csv"
    return f"{path}_{challenge_name}.{suffix}"


def save_results(
    output_path: str, title: str, output_format: OutputFormat, results: List[Results]
):
    challenge_output_file_path = build_path_for_challenge(
        output_path,
        re.sub(r"(?u)[^-\w.]", "", title.replace(" ", "_").replace(":", "_")),
        output_format,
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
    gamma: str = typer.Option(
        "1",
        help="""
        The gamma value applied for the calculation of the total stake threshold. It is applied to every single voting value before the sum is executed.
        """
    ),
    total_stake_threshold: float = typer.Option(
        0.01,
        help="""
        This value indicates the minimum percentage of voting needed by projects to be eligible for funding.
        Voting choices considered for this depends by the winner rule.
        """,
    ),
    relative_threshold: float = typer.Option(
        -1,
        help="This value indicates the relative threshold between Yes/No votes needed by projects to be eligible for funding.",
    ),
    output_format: OutputFormat = typer.Option("csv", help="Output format"),
    winner_selection_rule: WinnerSelectionRule = typer.Option(
        "yes_only",
        help="""
        The selection rule to apply to determine winner.
        Possible choices are:
        -   `yes_only` Fuzzy threshold voting: only YES votes are considered for ranking. Only YES votes are considered to calculate thresholds.
        -   `yes_no_diff` Fuzzy threshold voting: YES/NO difference is considered for ranking. Sum of YES/NO is considered to calculate thresholds.
        """,
    ),
    proposals_path: Optional[str] = typer.Option(None),
    excluded_proposals_path: Optional[str] = typer.Option(None),
    active_voteplan_path: Optional[str] = typer.Option(None),
    challenges_path: Optional[str] = typer.Option(None),
    vit_station_url: str = typer.Option("https://servicing-station.vit.iohk.io"),
    committee_keys_path: Optional[str] = typer.Option(None),
    fund_prefix: int = typer.Option(
        1100001,
        help="This number will be used to assign progressively project ids to winners.",
    ),
    leftovers_excluded_categories: List[int] = typer.Option(
        [],
        help="List of categories IDs that are not considered in leftovers winners calculation.",
    ),
    milestones_limit: List[int] = typer.Option(
        [0, 75000, 150000, 300000],
        help="Map of budgets to assign number of milestones. Lenght must coincide with `milestones_qty` parameter.",
    ),
    milestones_qty: List[int] = typer.Option(
        [3, 4, 5, 6],
        help="Map of milestones qty to assign number of milestones. Lenght must coincide with `milestones_limit` parameter.",
    ),
    ideascale_api_key: str = typer.Option(None, help="IdeaScale API key"),
    ideascale_api_url: str = typer.Option(
        "https://temp-cardano-sandbox.ideascale.com", help="IdeaScale API url"
    ),
    stage_ids: List[int] = typer.Option([], help="Stage IDs"),
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

    # minimum amount of stake needed for a proposal to be accepted
    _total_stake = calculate_total_stake_from_block0_configuration(
        block0_config, committee_keys, Fraction(1)
    )
    print(f"\nTotal stake before gamma applied {_total_stake}")
    print(f"Gamma as fractional exponent: {gamma}")

    total_stake = calculate_total_stake_from_block0_configuration(
        block0_config, committee_keys, Fraction(gamma)
    )

    total_stake_approval_threshold = float(total_stake_threshold) * float(total_stake)
    print(f"Total stake after gamma applied {total_stake}\n")

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
            winner_selection_rule,
            relative_threshold,
        )

        total_remaining_funds += remaining_funds
        all_results += results

        save_results(output_file, challenge.title, output_format, results)

    leftover_results, final_remaining_funds = calc_leftovers(
        all_results,
        total_remaining_funds,
        leftovers_excluded_categories,
        winner_selection_rule,
    )
    save_results(output_file, "leftovers", output_format, leftover_results)

    ideascale_proposals = []
    if ideascale_api_key:
        ideascale = IdeascaleImporter(ideascale_api_key, ideascale_api_url)

        async def _get_proposals():
            await ideascale.import_proposals(stage_ids=stage_ids)

        aiorun(_get_proposals())
        ideascale_proposals = ideascale.proposals

    milestones_limit.reverse()
    milestones_qty.reverse()
    winners = generate_winners(
        all_results + leftover_results,
        fund_prefix,
        milestones_limit,
        milestones_qty,
        ideascale_proposals,
    )
    save_results(output_file, "winners", output_format, winners)

    print("[bold green]Winners generated.[/bold green]")
    print(f"Total Stake: {total_stake}")
    print(f"Total Stake threshold: {total_stake_approval_threshold}")
    print(f"Leftover budget: {total_remaining_funds}")
    print(f"Unallocated budget: {final_remaining_funds}")
    print(f"Funded projects: {len(winners)}")


if __name__ == "__main__":
    typer.run(calculate_rewards)
