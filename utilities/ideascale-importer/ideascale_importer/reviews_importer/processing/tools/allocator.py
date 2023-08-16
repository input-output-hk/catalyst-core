"""Module to allocate proposals to PAs."""
from typing import List, Optional
from statistics import mean
import random
import rich
from loguru import logger

from ..types import models
from .. import utils
from .importer import Importer, IdeascaleImporter


class Allocator:
    """Perform the allocations."""

    def __init__(self, importer: Importer, nr_allocations: dict, seed: int, ideascale: Optional[IdeascaleImporter] = None):
        """Initialize entities."""
        self.data = importer
        if ideascale:
            self.ideascale = ideascale
            self.source = self.ideascale
        else:
            self.source = self.data

        self.nr_allocations = nr_allocations
        self.allocations: List[models.Allocation] = []
        random.seed(seed)

    def __get_relevant_proposals(
        self, challenge_ids: List[int], source: List[models.Proposal], level: int, times_check: bool = True
    ) -> List[models.Proposal]:
        times_picked = [len(p.allocations) for p in source]
        min_times_picked = min(times_picked)
        max_times_picked = max(times_picked)
        for x in range(min_times_picked, max_times_picked + 1):
            if len(challenge_ids) > 0:
                proposals = [p for p in source if (p.challenge_id in challenge_ids and (len(p.allocations) <= x and times_check))]
            else:
                proposals = [p for p in source if (len(p.allocations) <= x and times_check)]
            if len(proposals) >= self.nr_allocations[level]:
                return proposals
        return proposals

    def __generate_pa_challenges(self, pa: models.Pa, authors: dict) -> List[int]:
        """Generate challenges ids for each PA.

        Extract challenges as author.
        If level 1, use preferences as base challenges.
        If level 0, use all challenges as base challenges.
        Filter out challenges as author.
        If no challenges remain and pa is author in some challenge, use all challenges as base and filter the one as author.
        """
        if pa.id in authors.keys():
            challenges_as_author = authors[pa.id]
        else:
            challenges_as_author = []
        base_challenges = pa.challenge_ids if pa.level == 1 else []
        challenges_ids = list(filter(lambda c: c not in challenges_as_author, base_challenges))
        if len(challenges_ids) == 0 and len(challenges_as_author) > 0:
            challenges_ids = list(filter(lambda c: c not in challenges_as_author, [c.id for c in self.source.challenges]))
        return challenges_ids

    def __generate_authors_map(self):
        logger.info("Generating authors map...")
        authors = {}
        for proposal in self.source.proposals:
            for a in proposal.authors:
                if a.id in authors.keys():
                    if proposal.challenge_id not in authors[a.id]:
                        authors[a.id].append(proposal.challenge_id)
                else:
                    authors[a.id] = [proposal.challenge_id]
        return authors

    def __allocate_single(self, pa: models.Pa, amount: int, authors: dict, times_check: bool = True) -> List[models.Allocation]:
        pa_allocations = []
        challenge_ids = self.__generate_pa_challenges(pa, authors)
        relevant_proposals = self.__get_relevant_proposals(challenge_ids, self.source.proposals, pa.level, times_check)
        picked = random.sample(relevant_proposals, min(len(relevant_proposals), amount))
        for p in picked:
            allocation = models.Allocation(proposal=p, pa=pa)
            pa_allocations.append(allocation)
            p.allocations.append(allocation)
            pa.allocations.append(allocation)
        return pa_allocations

    def allocate(self):
        """Perform the allocation of proposals to PAs based on challenges' preference."""
        authors = self.__generate_authors_map()
        logger.info("Generating allocations...")
        for pa in self.source.pas:
            pa_allocations = self.__allocate_single(pa, self.nr_allocations[pa.level], authors)
            if len(pa_allocations) < self.nr_allocations[pa.level]:
                # When not enough allocations, allocate single again without distribution checks
                missing = self.nr_allocations[pa.level] - len(pa_allocations)
                additional_allocations = self.__allocate_single(pa, missing, authors, False)
                for a in additional_allocations:
                    if a not in pa_allocations:
                        pa_allocations.append(a)

            self.allocations = self.allocations + pa_allocations

    def generate_challenges_groups(self):
        """Generate challenges groups."""
        groups = []
        authors = self.__generate_authors_map()
        for challenge in self.source.challenges:
            group = {
                "challenge": utils.slugify(challenge.title),
                "pas": [],
            }
            for pa in self.source.pas:
                if pa.id not in authors:
                    group["pas"].append(pa)
                elif challenge.id not in authors[pa.id]:
                    group["pas"].append(pa)
            groups.append(group)
        return groups

    def allocation_stats(self):
        """Display stats for current allocation."""
        console = rich.console.Console()
        proposals_table = rich.table.Table("Type", "value", title="Proposals stats")
        proposals_picked = [len(p.allocations) for p in self.source.proposals]
        pas_picked = [len(p.allocations) for p in self.source.pas]
        proposals_table.add_row("max nr of allocations per proposal", str(max(proposals_picked)))
        proposals_table.add_row("min nr of allocations per proposal", str(min(proposals_picked)))
        proposals_table.add_row("avg nr of allocations per proposal", str(round(mean(proposals_picked), 2)))
        proposals_table.add_row("max nr of allocations per PAs", str(max(pas_picked)))
        proposals_table.add_row("min nr of allocations per PAs", str(min(pas_picked)))
        proposals_table.add_row("avg nr of allocations per PAs", str(round(mean(pas_picked), 2)))
        proposals_table.add_row("tot allocations", str(len(self.allocations)))
        proposals_table.add_row("tot proposals", str(len(self.source.proposals)))
        proposals_table.add_row("tot PAs", str(len(self.source.pas)))
        console.print(proposals_table)

    def export_allocations(self, source, path: str):
        """Export the produced allocation to CSV."""
        utils.deserialize_and_save_csv(
            path,
            source,
            {
                "anon_id": True,
                "name": True,
                "email": True,
                "level": True,
                "allocations": {"__all__": {"proposal": {"id", "url", "title", "challenge_id"}}},
            },
            "pa_allocation",
        )

    def export_single_allocations(self, source, path: str):
        """Export the single produced allocation to CSV."""
        utils.deserialize_and_save_csv(
            path,
            source,
            {
                "pa": {"anon_id": True, "email": True, "level": True},
                "proposal": {"id": True, "url": True, "title": True, "challenge_id": True},
            },
            "single_allocation",
        )
