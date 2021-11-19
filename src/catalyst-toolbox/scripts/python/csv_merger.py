import csv
import glob
from pathlib import Path
import os
from typing import List, Generator, Iterator, TextIO, Tuple
import typer
from contextlib import contextmanager


@contextmanager
def open_files(files: Iterator[str]) -> Generator[Iterator[TextIO], None, None]:
    files_objs = [open(file, encoding="utf-8", newline="") for file in files]
    yield iter(files_objs)
    for file in files_objs:
        file.close()


def search_file_pattern(pattern: str, base_path: Path) -> Iterator[str]:
    yield from map(
        lambda file: os.path.join(base_path, file),
        glob.iglob(pathname=pattern, root_dir=base_path),
    )


def file_as_csv(file: TextIO, delimiter: chr) -> Tuple[List[str], Iterator[List[str]]]:
    reader = csv.reader(file, delimiter=delimiter)
    return next(reader), reader


def merge_csv(
    pattern: str,
    base_path: Path,
    output_file: Path,
    input_delimiter: chr,
    output_delimiter: chr,
):
    print(pattern)
    files = search_file_pattern(pattern, base_path)
    with open(output_file, "w", encoding="utf-8", newline="") as out_file:
        with open_files(files) as fs:
            writer = csv.writer(out_file, delimiter=output_delimiter)
            header, first_content = file_as_csv(next(fs), delimiter=input_delimiter)
            writer.writerow(header)
            writer.writerows(first_content)
            for file in fs:
                # skip headers and use just content
                _, content = file_as_csv(file, delimiter=input_delimiter)
                writer.writerows(content)


def merge_csv_files(
    output_file: Path = typer.Option(...),
    pattern: str = typer.Option(...),
    base_path: Path = typer.Option(default=Path("./")),
    input_delimiter: str = typer.Option(default=","),
    output_delimiter: str = typer.Option(default=","),
):
    merge_csv(pattern, base_path, output_file, input_delimiter, output_delimiter)


if __name__ == "__main__":
    typer.run(merge_csv_files)
