"""Set of commands for the preparation of the review stage."""
import typer
import csv
from typing import List

from .tools.importer import Importer, IdeascaleImporter
from .tools.allocator import Allocator
from .tools.postprocessor import Postprocessor

app = typer.Typer()


async def allocate(
    pas_path: str,
    ideascale_api_key: str,
    ideascale_api_url: str,
    group_id: int, 
    challenges_group_id: int,
    stage_ids: List[int],
    nr_allocations: List[int],
    anonymize_start_id: int = 5000,
    seed: int = 7,
):
    """Run the allocator check."""

    async def _allocate():
        importer = Importer()
        ideascale = IdeascaleImporter(ideascale_api_key, ideascale_api_url)
        allocator = Allocator(importer, nr_allocations_map(), seed, ideascale)
        importer.load_pas(pas_path)
        await ideascale.import_challenges(challenges_group_id)
        await ideascale.import_proposals(stage_ids=stage_ids)
        await ideascale.import_com_revs(group_id=group_id, start_id=anonymize_start_id, historic_pas=importer.pas)
        allocator.allocate()
        # This data is not used by our core system, it is only consumed by Ideascale

        # allocator.export_allocations(allocator.source.pas, f"{output_path}/allocations-by-pa.xlsx")
        # groups = allocator.generate_challenges_groups()
        # for group in groups:
        #     allocator.export_allocations(group["pas"], f"{output_path}/{group['challenge']}.xlsx")

        allocator.allocation_stats()
        return allocator.allocations

    def nr_allocations_map():
        res = {}
        for i, el in enumerate(nr_allocations):
            res[i] = el
        return res

    return await _allocate()

async def process_ideascale_reviews(
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

    await _process_ideascale_reviews()
