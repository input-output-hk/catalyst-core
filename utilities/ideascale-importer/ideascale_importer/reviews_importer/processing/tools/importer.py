"""Module to import and manage data."""
from typing import List
from ..types import models
from .. import utils

from loguru import logger

import asyncio


class Importer:
    """Importer class."""

    def __init__(self):
        """Initialize entities."""
        self.proposals: List[models.Proposal] = []
        self.challenges: List[models.Challenge] = []
        self.pas: List[models.Pa] = []
        self.reviews: List[models.Review] = []
        self.allocations: List[models.AllocationLight] = []

    def load_proposals(self, path: str):
        """Load a list of proposals from a json file."""
        self.proposals = utils.load_json_and_serialize(path, models.Proposal)

    def load_challenges(self, path: str):
        """Load a list of challenges from a json file."""
        self.challenges = utils.load_json_and_serialize(path, models.Challenge)

    def load_pas(self, path: str):
        """Load a list of PAs from a csv file."""
        self.pas = utils.load_csv_and_serialize(path, models.Pa, {"challenges": self.challenges})

    def load_reviews(self, path: str, fund: int):
        """Load a list of reviews from a csv file."""
        self.reviews = self.reviews + utils.load_csv_and_serialize(path, models.Review, {"fund": fund})

    def load_allocations(self, path: str, fund: int):
        """Load a list of allocation from a csv file."""
        self.allocations = self.allocations + utils.load_csv_and_serialize(path, models.AllocationLight, {"fund": fund})

    def prepare_reviews(self, criteria) -> tuple[List[str], List[dict]]:
        """Prepare reviews as a list of texts and mappings given a set of criteria."""
        notes: List[str] = []
        mapping: List[dict] = []
        for review in self.reviews:
            for criterium in criteria:
                notes.append(getattr(review, criterium))
                mapping.append({"review": review, "criterium": criterium})

        return notes, mapping


class IdeascaleImporter:
    """Interface with IdeaScale API."""

    def __init__(self, api_key: str, api_url: str = "https://temp-cardano-sandbox.ideascale.com"):
        """Initialize entities."""
        self.api_key = api_key
        self.api_url = api_url
        self.inner = utils.JsonHttpClient(self.api_url)
        self.N_WORKERS = 6

        self.challenges: List[models.Challenge] = []
        self.proposals: List[models.Proposal] = []
        self.pas: List[models.Pas] = []

    def update_comrevs_emails(self, historic_pas: List[models.Pa] = []):

        self.new_pas = []
        async def wrapped():
            tasks: asyncio.Queue = asyncio.Queue()
            for pa in historic_pas:
                tasks.put_nowait(self.get_user(pa))

            async def worker():
                while not tasks.empty():
                    await tasks.get_nowait()

            await asyncio.gather(*[worker() for _ in range(self.N_WORKERS)])

        asyncio.run(wrapped())

    async def get_user(self, pa: models.Pa):
        try:
            res = await self._get(f"/a/rest/v1/members/email/{pa.email}")
            new_pa = {
                'old': pa.email,
                'new': res['email']
            }
            self.new_pas.append(new_pa)
        except utils.GetFailed:
            new_pa = {
                'old': pa.email,
                'new': ''
            }
            self.new_pas.append(new_pa)
        except:
            new_pa = {
                'old': pa.email,
                'new': ''
            }
            self.new_pas.append(new_pa)


    async def import_com_revs(
        self, group_id: int, page_size: int = 50, start_id: int = 0, historic_pas: List[models.Pa] = []
    ) -> List[models.IdeascaleComRev]:
        """Import from Ideascale the registered ComRevs."""

        class WorkerData:
            page: int = 0
            done: bool = False
            reviewers: List[models.IdeascaleComRev] = []

        async def worker(d: WorkerData):
            while True:
                if d.done:
                    break

                p = d.page
                d.page += 1

                res = await self._get(f"/a/rest/v1/groups/{group_id}/members/{p}/{page_size}")

                res_reviewers: List[models.IdeascaleComRev] = []
                for i in res:
                    res_reviewers.append(models.IdeascaleComRev(**i))

                d.reviewers.extend(res_reviewers)

                if len(res_reviewers) < page_size:
                    d.done = True

        d = WorkerData()
        worker_tasks = [asyncio.create_task(worker(d)) for _ in range(self.N_WORKERS)]
        for task in worker_tasks:
            await task
        reviewers = list(filter(lambda r: r.subscribed, d.reviewers))
        self.transform_pas(reviewers, historic_pas, start_id)

    async def import_challenges(self, group_id: int):
        """Import from Ideascale the Challenges."""
        res = await self._get(f"/a/rest/v1/campaigns/groups/{group_id}")

        challenges: List[models.IdeascaleChallenge] = []
        for group in res:
            assert isinstance(group, dict)

            if "campaigns" in group:
                group_challenges = []
                for c in group["campaigns"]:
                    group_challenges.append(models.IdeascaleChallenge(**c))
                    await asyncio.sleep(0)

                challenges.extend(group_challenges)

        self.challenges = [models.Challenge(**c.dict()) for c in challenges]

    async def import_proposals(self, stage_ids: List[int], campaign_id: int = None, page_size: int = 50):
        """Get all ideas from the stage with the given id.

        Pages are requested concurrently until the latest one fails
        which signals that that are no more pages left.
        """

        class WorkerData:
            def __init__(self, stage_id):
                self.stage_id = stage_id

                self.page: int = 0
                self.done: bool = False
                self.proposals: List[models.Proposal] = []

        async def worker(d: WorkerData, stage_id: int):
            while True:
                if d.done:
                    break

                p = d.page
                d.page += 1

                if campaign_id is not None:
                    res = await self._get(f"/a/rest/v1/campaigns/{campaign_id}/ideas/{p}/{page_size}")
                else:
                    res = await self._get(f"/a/rest/v1/stages/{stage_id}/ideas/{p}/{page_size}")

                res_proposals: List[models.Proposal] = []
                for i in res:
                    if i["stage_id"] == stage_id:
                        res_proposals.append(models.Proposal(**i))

                d.proposals.extend(res_proposals)

                if len(res_proposals) < page_size:
                    d.done = True
        d = {}
        for stage_id in stage_ids: 
            d = WorkerData(stage_id)
            worker_tasks = [asyncio.create_task(worker(d, stage_id)) for _ in range(self.N_WORKERS)]
            for task in worker_tasks:
                await task
            self.proposals.extend(d.proposals)

    def change_proposals_stage(self, target_stage: int):
        """Change proposals stage."""

        logger.info(f"Number of proposal to move: {len(self.proposals)}")

        async def wrapped():
            tasks: asyncio.Queue = asyncio.Queue()
            for proposal in self.proposals:
                tasks.put_nowait(self._change_proposal_stage(proposal, target_stage))

            async def worker():
                while not tasks.empty():
                    await tasks.get_nowait()

            await asyncio.gather(*[worker() for _ in range(self.N_WORKERS)])

        asyncio.run(wrapped())
    
    def change_proposals_campaign(self, target_campaign: int, target_stage: int):
        """Change proposals stage."""

        logger.info(f"Number of proposal to move: {len(self.proposals)}")

        async def wrapped():
            tasks: asyncio.Queue = asyncio.Queue()
            for proposal in self.proposals:
                tasks.put_nowait(self._change_proposal_campaign(proposal, target_campaign, target_stage))

            async def worker():
                while not tasks.empty():
                    await tasks.get_nowait()

            await asyncio.gather(*[worker() for _ in range(self.N_WORKERS)])

        asyncio.run(wrapped())

    async def _change_proposal_stage(self, proposal: models.Proposal, target_stage: int):
        """Change proposal stage."""
        await self._post(f"/a/rest/v1/ideas/{proposal.id}/changeStage/{target_stage}")

    async def _change_proposal_campaign(self, proposal: models.Proposal, target_campaign: int, target_stage: int):
        """Change proposal stage."""
        await self._post(f"/a/rest/v1/ideas/{proposal.id}/changeCampaign/{target_campaign}/targetStage/{target_stage}")

    async def _get(self, path: str):
        """Execute a GET request."""
        headers = {"api_token": self.api_key}
        return await self.inner.get(path, headers)

    async def _post(self, path: str, data: dict = None):
        """Execute a POST request."""
        headers = {"api_token": self.api_key}
        return await self.inner.post(path, headers=headers)

    def transform_pas(self, reviewers: List[models.IdeascaleComRev], historic_pas: List[models.Pa], start_id: int = 0):
        """Merge historic reviewers with the new ones and assign level and challenges accordingly."""
        challenges_map = {}
        for challenge in self.challenges:
            challenges_map[challenge.title] = challenge
        historic_pas_map = {}
        for historic_pa in historic_pas:
            historic_pas_map[historic_pa.email] = historic_pa

        for r in reviewers:
            if r.email in historic_pas_map.keys():
                # We store preferred challenges as string in Ideascale so we need to convert them to list of ids
                reviewer_challenges = [challenges_map[c] for c in r.preferred_challenges]
                level = 1
            else:
                reviewer_challenges = []
                level = 0

            self.pas.append(models.Pa(**r.dict(), ids=str(start_id), level=level, challenges=reviewer_challenges))
            start_id = start_id + 1

    def _find_challenge(self, challenge_title):
        for c in self.challenges:
            if c.title == challenge_title:
                return c
        return None

    def raw_reviews_from_file(self, path: str):
        """Import reviews from a xlsx file."""
        logger.info("Unmerging cells...")
        wb = utils.unmerge_xlsx(path)
        logger.info("Extract assessments...")
        reviews = utils.get_rows_from_xlsx(wb, "Assessments")
        logger.info("Extract assessments result...")
        _results = utils.get_rows_from_xlsx(wb, "IdeaScale - Assessment Results")
        logger.info("Parse assessments...")
        reviews = [models.IdeascaleExportedReview(**r) for r in filter(lambda r: r["Assessor"] is not None, reviews)]
        logger.info("Parse assessments result...")
        _results = [models.IdeascaleExportedReviewResult(**r) for r in _results]
        logger.info("Associate idea ID...")
        _cache_results = list(_results)
        for r in reviews:
            _related = next(
                (_r for _r in _cache_results if (_r.email == r.email and _r.idea_title == r.idea_title and _r.date == r.date)), None
            )
            if _related is None:
                logger.error("File malformed...")
            r.idea_id = _related.idea_id
            r.idea_title = _related.idea_title
            r.idea_challenge = self._find_challenge(_related.campaign_title)

        return reviews

    def group_triplets(self, _reviews):
        """Given a list of reviews divided by criteria, group them for the complete review."""
        groups = {}
        questions = {
            "This proposal effectively addresses the challenge": "Impact / Alignment",
            "Given experience and plan presented it is highly likely this proposal will be implemented successfully": "Feasibility",
            "The information provided is sufficient to audit the progress and the success of the proposal.": "Auditability",
        }
        logger.info("Group triplets...")
        for review in _reviews:
            key = f"{review.idea_id}-{review.email}"
            if review.question in questions:
                if key not in groups:
                    groups[key] = {}
                groups[key][questions[review.question]] = review

        reviews = []
        logger.info("Parse reviews...")
        _questions = list(questions.values())
        for idx, g in enumerate(groups.keys()):
            if len(groups[g].keys()) == 3:
                triplet = groups[g]
                review_dict = {
                    "id": idx,
                    "Assessor": triplet[_questions[0]].email,
                    "Impact / Alignment Note": triplet[_questions[0]].note,
                    "Impact / Alignment Rating": triplet[_questions[0]].score,
                    "Feasibility Note": triplet[_questions[1]].note,
                    "Feasibility Rating": triplet[_questions[1]].score,
                    "Auditability Note": triplet[_questions[2]].note,
                    "Auditability Rating": triplet[_questions[2]].score,
                }
                proposal_dict = {
                    "id": triplet[_questions[0]].idea_id,
                    "url": triplet[_questions[0]].idea_url,
                    "title": triplet[_questions[0]].idea_title,
                    "campaign_id": triplet[_questions[0]].idea_challenge.id,
                }
                reviews.append(models.Review(**review_dict, proposal=models.Proposal(**proposal_dict)))
        return reviews

    def extract_proposers_emails(self):
        proposers_emails = []
        public_emails = []
        for proposal in self.proposals:
            proposers_emails = proposers_emails + [{"email": a.email} for a in proposal.authors]
            public_emails.append({"email": proposal.public_email})

        return proposers_emails, public_emails
