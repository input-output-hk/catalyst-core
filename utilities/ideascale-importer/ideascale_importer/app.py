"""CLI entrypoint."""

import typer
import ideascale_importer.cli.ideascale
import ideascale_importer.cli.snapshot
import ideascale_importer.cli.reviews

app = typer.Typer(add_completion=False)
app.add_typer(ideascale_importer.cli.ideascale.app, name="ideascale", help="IdeaScale commands (e.g. importing data)")
app.add_typer(ideascale_importer.cli.snapshot.app, name="snapshot", help="Snapshot commands (e.g. importing data)")
app.add_typer(ideascale_importer.cli.reviews.app, name="reviews", help="Reviews commands (e.g. importing data)")


if __name__ == "__main__":
    app()
