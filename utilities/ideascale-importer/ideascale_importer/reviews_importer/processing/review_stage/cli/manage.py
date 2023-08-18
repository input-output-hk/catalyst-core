"""Set of commands to manage ideascale."""
import typer
import csv
from typing import List
from rich import print
from asyncio import run as aiorun

from review_stage.tools.importer import IdeascaleImporter

app = typer.Typer()


@app.command()
def move_stages(
    ideascale_api_key: str = typer.Option("", help="IdeaScale API key"),
    ideascale_api_url: str = typer.Option("https://temp-cardano-sandbox.ideascale.com", help="IdeaScale API url"),
    source_stage_id: int = typer.Option(1, help="Source stage ID"),
    target_stage_id: int = typer.Option(1, help="Target stage ID"),
):
    """Run the script to move proposals from stage to stage."""

    ideascale = IdeascaleImporter(ideascale_api_key, ideascale_api_url)

    async def _get_proposals():
        await ideascale.import_proposals(stage_ids=[source_stage_id])
        print("[bold green]Proposals loaded.[/bold green]")

    aiorun(_get_proposals())
    ideascale.change_proposals_stage(target_stage_id)
    print("[bold green]Proposals stage changed.[/bold green]")


@app.command()
def move_campaigns(
    ideascale_api_key: str = typer.Option("", help="IdeaScale API key"),
    ideascale_api_url: str = typer.Option("https://temp-cardano-sandbox.ideascale.com", help="IdeaScale API url"),
    source_campaign_id: int = typer.Option(1, help="Source campaign ID"),
    target_campaign_id: int = typer.Option(1, help="Target campaign ID"),
    source_stage_id: int = typer.Option(1, help="Source stage ID"),
    target_stage_id: int = typer.Option(1, help="Target stage ID"),
):
    """Run the script to move proposals from stage to stage."""

    ideascale = IdeascaleImporter(ideascale_api_key, ideascale_api_url)

    async def _get_proposals():
        await ideascale.import_proposals(campaign_id=source_campaign_id, stage_ids=[source_stage_id])
        print("[bold green]Proposals loaded.[/bold green]")

    aiorun(_get_proposals())
    ideascale.change_proposals_campaign(target_campaign_id, target_stage_id)
    print("[bold green]Proposals campaign changed.[/bold green]")


@app.command()
def archive_proposals_by_keywords(
    ideascale_api_key: str = typer.Option("", help="IdeaScale API key"),
    ideascale_api_url: str = typer.Option("https://temp-cardano-sandbox.ideascale.com", help="IdeaScale API url"),
    source_stage_id: int = typer.Option(1, help="Source stage ID"),
    archive_stage_id: int = typer.Option(1, help="Target stage ID"),
    kw: List[str] = typer.Option([""], help="Keywords to filter proposals"),
):
    ideascale = IdeascaleImporter(ideascale_api_key, ideascale_api_url)

    async def _get_proposals():
        await ideascale.import_proposals(stage_ids=[source_stage_id])
        print("[bold green]Proposals loaded.[/bold green]")

    aiorun(_get_proposals())
    for p in ideascale.proposals:
        if any(k.lower() in p.full_text.lower() for k in kw):
            archive = typer.confirm(f"Are you sure you want to archive {p.title} - {p.url}?")
            if archive:
                ideascale._change_proposal_stage(p, archive_stage_id)


@app.command()
def archive_proposals_by_budget(
    ideascale_api_key: str = typer.Option("", help="IdeaScale API key"),
    ideascale_api_url: str = typer.Option("https://temp-cardano-sandbox.ideascale.com", help="IdeaScale API url"),
    source_campaign_id: int = typer.Option(1, help="Source campaign ID"),
    source_stage_id: int = typer.Option(1, help="Source stage ID"),
    archive_stage_id: int = typer.Option(1, help="Target stage ID"),
    max_budget: int = typer.Option(0, help="Max budget allowed"),
    min_budget: int = typer.Option(0, help="Min budget allowed"),
):
    ideascale = IdeascaleImporter(ideascale_api_key, ideascale_api_url)

    async def _get_proposals():
        await ideascale.import_proposals(campaign_id=source_campaign_id, stage_ids=[source_stage_id])
        print(f"[bold green]{len(ideascale.proposals)} Proposals loaded.[/bold green]")

    aiorun(_get_proposals())
    for p in ideascale.proposals:
        if p.funds > max_budget or p.funds < min_budget:
            archive = typer.confirm(f"Are you sure you want to archive {p.title} - {p.url}?")
            if archive:
                ideascale._change_proposal_stage(p, archive_stage_id)


@app.command()
def export_emails(
    ideascale_api_key: str = typer.Option("", help="IdeaScale API key"),
    ideascale_api_url: str = typer.Option("https://temp-cardano-sandbox.ideascale.com", help="IdeaScale API url"),
    stage_ids: List[int] = typer.Option([], help="Stage ID"),
):
    ideascale = IdeascaleImporter(ideascale_api_key, ideascale_api_url)

    async def _get_proposals():
        await ideascale.import_proposals(stage_ids=stage_ids)
        print("[bold green]Proposals loaded.[/bold green]")

    aiorun(_get_proposals())
    proposers_emails, public_emails = ideascale.extract_proposers_emails()
