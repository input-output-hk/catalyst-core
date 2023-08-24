"""Set of commands for the preparation of the review stage."""
import typer
import csv
from typing import List, Dict

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
    output_path: str,
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

        file_name = f"{output_path}/allocations.csv"
        allocator.export_single_allocations(allocator.allocations, file_name)
        allocator.allocation_stats()
        return file_name

    def nr_allocations_map():
        res = {}
        for i, el in enumerate(nr_allocations):
            res[i] = el
        return res

    return await _allocate()

async def process_ideascale_reviews(
    ideascale_xlsx_path: List[str],
    ideascale_api_key: str,
    ideascale_api_url: str,
    allocation_path: str,
    challenges_group_id: int,
    questions: Dict[str, str],
    output_path: str,
    fund: int,
):
    """Process Ideascale export."""

    async def _process_ideascale_reviews():
        importer = Importer()
        postprocessor = Postprocessor(importer)
        importer.load_allocations(allocation_path, fund)
        ideascale = IdeascaleImporter(ideascale_api_key, ideascale_api_url)
        await ideascale.import_challenges(challenges_group_id)
        start_id = 1
        for xlsx in ideascale_xlsx_path:
            if len(importer.reviews) > 0:
                start_id = importer.reviews[-1].id + 1
            reviews = ideascale.raw_reviews_from_file(xlsx)
            reviews = ideascale.group_triplets(reviews, questions, start_id=start_id)
            importer.reviews = importer.reviews + reviews

        postprocessor.postprocess_reviews()
        postprocessor.export_reviews(postprocessor.data.reviews, f"{output_path}/postprocessed-reviews.csv")
        postprocessor.export_pas(f"{output_path}/active-pas.csv")

    await _process_ideascale_reviews()
