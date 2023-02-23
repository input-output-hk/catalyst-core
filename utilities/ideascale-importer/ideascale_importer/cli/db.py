import asyncio
from datetime import datetime
import random
import rich
import typer

import ideascale_importer.db
from ideascale_importer.db import models


app = typer.Typer(add_completion=False)


@app.command()
def seed_compatible(database_url: str = typer.Option(..., help="Postgres database URL")):
    """
    Generate seed data that is compatible with the old vit-servicing-station schema
    """

    async def inner(database_url: str):
        console = rich.console.Console()

        conn = await ideascale_importer.db.connect(database_url)
        console.log("Connected to database")

        async with conn.transaction():
            election = models.Election(
                name="Fund 10",
                description="Fund 10 event",
                registration_snapshot_time=datetime.now(),
                voting_power_threshold=1,
                max_voting_power_pct=50,
                start_time=datetime.now(),
                end_time=datetime.now(),
                insight_sharing_start=datetime.now(),
                proposal_submission_start=datetime.now(),
                refine_proposals_start=datetime.now(),
                finalize_proposals_start=datetime.now(),
                proposal_assessment_start=datetime.now(),
                assessment_qa_start=datetime.now(),
                snapshot_start=datetime.now(),
                voting_start=datetime.now(),
                voting_end=datetime.now(),
                tallying_end=datetime.now(),
                extra={
                    "url": {
                        "results": "https://election.com/results/10",
                        "survey": "https://election.com/survey/10",
                    }
                })
            election_id = await conn.insert(election, returning="row_id")
            console.log(f"Inserted election row_id={election_id}")

            voting_group = models.VotingGroup(group_id="group-id-1", election_id=election_id, token_id="token-id-1")
            voting_group_row_id = await conn.insert(voting_group, returning="row_id")
            console.log(f"Inserted voting_group row_id={voting_group_row_id}")

            voteplan = models.Voteplan(election_id=election_id, id="voteplan-1", category="public",
                                       encryption_key="encryption-key-1", group_id=voting_group_row_id)
            voteplan_row_id = await conn.insert(voteplan, returning="row_id")
            console.log(f"Inserted voteplan row_id={voteplan_row_id}")

            for i in range(2):
                challenge_id = random.randint(1, 1000)

                challenge = models.Challenge(
                    id=challenge_id,
                    election=election_id,
                    category="simple",
                    title=f"Challenge {i}",
                    description=f"Random challenge {i}",
                    rewards_currency="ADA",
                    rewards_total=100000,
                    proposers_rewards=10000,
                    vote_options=await conn.get_vote_options_id("yes,no"),
                    extra={
                        "url": {
                            "challenge": f"https://challenge.com/{i}"
                        },
                        "highlights": {
                            "sponsor": f"Highlight {i} sponsor",
                        },
                    })

                challenge_row_id = await conn.insert(challenge, returning="row_id")
                console.log(f"Inserted challenge row_id={challenge_row_id}")

                for j in range(2):
                    proposal_id = random.randint(1, 1000)

                    proposal = models.Proposal(
                        id=proposal_id,
                        challenge=challenge_id,
                        title=f"Proposal {i}-{j}",
                        summary=f"Random proposal {i}-{j}",
                        category=f"Random category {i}-{j}",
                        public_key="",
                        funds=10000,
                        url=f"https://somewhere.com/proposal/{i}-{j}",
                        files_url="",
                        impact_score=random.randint(100, 400),
                        extra={"solution": f"Solution {i}-{j}"},
                        proposer_name=f"Proposer {i}-{j}",
                        proposer_contact="",
                        proposer_url=f"https://proposer-{i}-{j}.com",
                        proposer_relevant_experience="",
                        bb_proposal_id=b"someid",
                        bb_vote_options="yes,no"
                    )

                    proposal_row_id = await conn.insert(proposal, returning="row_id")
                    console.log(f"Inserted proposal row_id={proposal_row_id}")

                    proposal_voteplan = models.ProposalVoteplan(
                        proposal_id=proposal_row_id, voteplan_id=voteplan_row_id, bb_proposal_index=(i+1)*(j+1))
                    await conn.insert(proposal_voteplan)

            for i in range(1):
                goal = models.Goal(election_id=election_id, idx=i, name=f"Goal {i}")
                goal_id = await conn.insert(goal, returning="id")
                console.log(f"Inserted goal id={goal_id}")

    asyncio.run(inner(database_url))
