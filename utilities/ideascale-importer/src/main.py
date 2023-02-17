import typer
import cli.db
import cli.ideascale


app = typer.Typer(add_completion=False)
app.add_typer(cli.db.app, name="db", help="Postgres DB commands (e.g. seeding)")
app.add_typer(cli.ideascale.app, name="ideascale", help="IdeaScale commands (e.g. importing data)")

if __name__ == "__main__":
    app()
