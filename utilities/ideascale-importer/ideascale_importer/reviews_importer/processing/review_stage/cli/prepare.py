"""Set of commands for the preparation of the review stage."""
import typer
import csv
from typing import List
from rich import print
from asyncio import run as aiorun

from review_stage.tools.importer import Importer, IdeascaleImporter
from review_stage.tools.allocator import Allocator
from review_stage.tools.postprocessor import Postprocessor
from review_stage.tools.sql_generator import SQLGenerator

app = typer.Typer()


@app.command()
def allocate(
    nr_allocations: List[int] = typer.Option([30, 80], help="Nr of proposal to allocate"),
    seed: int = typer.Option(7, help="Seed number for random allocations"),
    proposals_path: str = typer.Option("", help="Proposals file"),
    challenges_path: str = typer.Option("", help="Challenges file"),
    pas_path: str = typer.Option("", help="PAs file"),
    ideascale_api_key: str = typer.Option("", help="IdeaScale API key"),
    ideascale_api_url: str = typer.Option("https://temp-cardano-sandbox.ideascale.com", help="IdeaScale API url"),
    group_id: int = typer.Option(1, help="Reviewers group ID"),
    challenges_group_id: int = typer.Option(1, help="Challenges group ID"),
    anonymize_start_id: int = typer.Option(5000, help="ID to start with to assign anonymized IDs"),
    stage_ids: List[int] = typer.Option([], help="Stage ID"),
    output_path: str = typer.Option("", help="Output folder"),
):
    """Run the allocator check."""

    async def _allocate():
        importer = Importer()
        ideascale = IdeascaleImporter(ideascale_api_key, ideascale_api_url)
        allocator = Allocator(importer, nr_allocations_map(), seed, ideascale)
        importer.load_pas(pas_path)
        await ideascale.import_challenges(challenges_group_id)
        await ideascale.import_com_revs(group_id=group_id, start_id=anonymize_start_id, historic_pas=importer.pas)
        await ideascale.import_proposals(stage_ids=stage_ids)
        allocator.allocate()
        allocator.export_allocations(allocator.source.pas, f"{output_path}/allocations-by-pa.xlsx")
        groups = allocator.generate_challenges_groups()
        for group in groups:
            allocator.export_allocations(group["pas"], f"{output_path}/{group['challenge']}.xlsx")
        allocator.export_single_allocations(allocator.allocations, f"{output_path}/allocations.csv")
        allocator.allocation_stats()
        print("[bold green]Proposals allocated.[/bold green]")

    def nr_allocations_map():
        res = {}
        for i, el in enumerate(nr_allocations):
            res[i] = el
        return res

    aiorun(_allocate())

@app.command()
def update_emails(
    pas_path: str = typer.Option("", help="PAs file"),
    ideascale_api_key: str = typer.Option("", help="IdeaScale API key"),
    ideascale_api_url: str = typer.Option("https://temp-cardano-sandbox.ideascale.com", help="IdeaScale API url"),
    output_path: str = typer.Option("", help="Output file"),
):
    
    importer = Importer()
    ideascale = IdeascaleImporter(ideascale_api_key, ideascale_api_url)
    
    async def _update_emails():
        
        importer.load_pas(pas_path)
    
    aiorun(_update_emails())
    ideascale.update_comrevs_emails(historic_pas=importer.pas)
    keys = ideascale.new_pas[0].keys()
    with open(output_path, "w", newline="") as f:
        dict_writer = csv.DictWriter(f, keys)
        dict_writer.writeheader()
        dict_writer.writerows(ideascale.new_pas)

@app.command()
def process_ideascale_reviews(
    ideascale_xlsx_path: str = typer.Option("", help="Review export from Ideascale."),
    ideascale_api_key: str = typer.Option("", help="IdeaScale API key"),
    ideascale_api_url: str = typer.Option("https://temp-cardano-sandbox.ideascale.com", help="IdeaScale API url"),
    allocation_path: str = typer.Option("", help="Allocation file"),
    challenges_group_id: int = typer.Option(1, help="Challenges group ID"),
    output_path: str = typer.Option("", help="Output folder"),
    fund: int = typer.Option(10, help="Fund"),
):
    """Process Ideascale export."""

    async def _process_ideascale_reviews():
        importer = Importer()
        postprocessor = Postprocessor(importer)
        importer.load_allocations(allocation_path, fund)
        ideascale = IdeascaleImporter(ideascale_api_key, ideascale_api_url)
        await ideascale.import_challenges(challenges_group_id)
        reviews = ideascale.raw_reviews_from_file(ideascale_xlsx_path)
        reviews = ideascale.group_triplets(reviews)
        importer.reviews = reviews

        postprocessor.postprocess_reviews()
        postprocessor.export_reviews(postprocessor.data.reviews, f"{output_path}/postprocessed-reviews.csv")
        postprocessor.export_pas(f"{output_path}/active-pas.csv")

    aiorun(_process_ideascale_reviews())


@app.command()
def generate_sqls(
    ideascale_api_key: str = typer.Option("", help="IdeaScale API key"),
    ideascale_api_url: str = typer.Option("https://temp-cardano-sandbox.ideascale.com", help="IdeaScale API url"),
    challenges_group_id: int = typer.Option(1, help="Challenges group ID"),
    fund: int = typer.Option(10, help="Fund ID"),
    stage_ids: int = typer.Option([], help="Stage ID"),
    reviews_path: str = typer.Option("", help="Active PAs file"),
    output_path: str = typer.Option("", help="Output folder"),
):
    """Generate SQLs for the moderation backend."""

    async def _generate_sqls():
        importer = Importer()
        ideascale = IdeascaleImporter(ideascale_api_key, ideascale_api_url)
        sql_generator = SQLGenerator(importer, ideascale, output_path)
        sql_generator.funds(fund)
        await sql_generator.challenges(fund, challenges_group_id)
        await sql_generator.proposals(fund, stage_ids)
        sql_generator.reviews(reviews_path, fund)

    aiorun(_generate_sqls())
