import typer
import ideascale_importer.cli.db
import ideascale_importer.cli.ideascale


app = typer.Typer(add_completion=False)
app.add_typer(ideascale_importer.cli.db.app, name="db", help="Postgres DB commands (e.g. seeding)")
app.add_typer(ideascale_importer.cli.ideascale.app, name="ideascale", help="IdeaScale commands (e.g. importing data)")

app()
