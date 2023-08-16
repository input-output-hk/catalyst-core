"""Set of commands for review analysis."""
from typing import List
import typer
from rich import print

from review_stage.tools.importer import Importer
from review_stage.tools.similarity import Similarity
from review_stage.tools.profanity import Profanity
from review_stage.tools.gptzero import GptZero

app = typer.Typer()


@app.command()
def similarity(
    proposals_path: str = typer.Option("", help="Proposals file"),
    challenges_path: str = typer.Option("", help="Challenges file"),
    pas_path: str = typer.Option("", help="PAs file"),
    reviews_path: List[str] = typer.Option([], help="Reviews files"),
    funds_map: List[int] = typer.Option([], help="Funds ids"),
    current_fund: int = typer.Option(9, help="Current Fund"),
    similarity_threshold: float = typer.Option(0.99, help="The threshold used to detect similarity."),
    output_path: str = typer.Option("", help="Output file"),
):
    """Run the similarity check."""
    importer = Importer()
    similarity = Similarity(importer, similarity_threshold)
    importer.load_proposals(proposals_path)
    importer.load_challenges(challenges_path)
    importer.load_pas(pas_path)
    for idx, p in enumerate(reviews_path):
        importer.load_reviews(p, funds_map[idx])

    results = similarity.generate_similarity(current_fund)
    similarity.export_results(output_path, results)

    print("[bold green]Reviews similarity generated.[/bold green]")


@app.command()
def profanity(
    reviews_path: str = typer.Option("", help="Reviews files"),
    current_fund: int = typer.Option(9, help="Current Fund"),
    profanity_threshold: float = typer.Option(0.75, help="The threshold used to detect profanities."),
    output_path: str = typer.Option("", help="Output file"),
):
    """Run the similarity check."""
    importer = Importer()
    profanity = Profanity(importer, profanity_threshold)
    importer.load_reviews(reviews_path, current_fund)
    results = profanity.check_profanity()
    profanity.export_results(output_path, results)


@app.command()
def detect_ai(
    reviews_path: str = typer.Option("", help="Reviews files"),
    current_fund: int = typer.Option(9, help="Current Fund"),
    api_key: str = typer.Option("", help="GptZero API key"),
    ai_threshold: float = typer.Option(0.65, help="The threshold used to detect ai generate content."),
    output_path: str = typer.Option("", help="Output file"),
):
    """Run the similarity check."""
    importer = Importer()
    gptzero = GptZero(importer, api_key, ai_threshold)
    importer.load_reviews(reviews_path, current_fund)
    gptzero.detect_by_reviewer()
    results = gptzero.detect_ai()
    gptzero.export_results(output_path, results)
