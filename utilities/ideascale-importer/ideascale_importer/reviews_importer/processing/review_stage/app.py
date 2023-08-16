"""Main module for review stage."""
import typer
from review_stage.cli import analysis
from review_stage.cli import prepare
from review_stage.cli import manage

app = typer.Typer()

app.add_typer(analysis.app, name="analysis", help="Set of commands for the analysis of F10 review stage.")
app.add_typer(prepare.app, name="prepare", help="Set of commands for the preparation of F10 review stage.")
app.add_typer(manage.app, name="manage", help="Set of commands for the management of Ideascale.")

if __name__ == "__main__":
    app()
