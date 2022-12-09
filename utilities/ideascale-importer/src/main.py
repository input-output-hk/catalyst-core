import typer

import cli.ideascale
import cli.postgres

app = typer.Typer(add_completion=False)
app.add_typer(cli.ideascale.app, name="ideascale", help="Commands for calling IdeaScale API")
app.add_typer(cli.postgres.app, name="postgres", help="Commands for importing data to Postgres")

if __name__ == "__main__":
    app()
