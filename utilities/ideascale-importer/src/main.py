import typer

import cli.ideascale

app = typer.Typer()
app.add_typer(cli.ideascale.app, name="ideascale", help="IdeaScale API commands")

if __name__ == "__main__":
    app()
