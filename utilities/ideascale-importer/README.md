IdeaScale Importer
===

## Getting Started

First install [pyenv](https://github.com/pyenv/pyenv#installation) AND [pyenv virtualenv](https://github.com/pyenv/pyenv-virtualenv).

Then create a virtual environment and install dependencies:

```sh
pyenv install 3.11.1
pyenv virtualenv 3.11.1 ideascale-importer-venv-3.11.1
pyenv  activate ideascale-importer-venv-3.11.1
pip install -r requirements.txt
```

To see the available commands:

```sh
python src/main.py --help
```

## Importing IdeaScale Data

The easiest way is to run:

```sh
python src/main.py \
    --api-token IDEASCALE_API_TOKEN \
    --database-url POSTGRES_URL
```

And go through the interactive steps.

## Development

### Linting

```sh
# If you haven't already:
python -m flake8 src
```

### Type checking

```sh
python -m mypy src --check-untyped-defs
```
