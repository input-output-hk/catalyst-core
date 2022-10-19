#!/usr/bin/env python3

from pathlib import Path
import subprocess
import sys
import shutil
from directories import repository_directory, plugin_directory


def run():
    if (
        subprocess.run(
            [
                "uniffi-bindgen",
                "generate",
                "-l",
                "kotlin",
                "src/lib.udl",
                "--config",
                "uniffi.toml",
                "-o",
                "codegen/kotlin",
            ],
            cwd=repository_directory / "bindings" / "wallet-uniffi",
        ).returncode
        != 0
    ):
        print("couldn't build kotlin bindings")
        sys.exit(1)

    src_files = (
        repository_directory
        / "bindings"
        / "wallet-uniffi"
        / "codegen"
        / "kotlin"
        / "com"
        / "iohk"
        / "jormungandr_wallet"
    ).glob("*kt")

    dst = plugin_directory / Path("src/android/")
    dst.mkdir(parents=True, exist_ok=True)

    print("Copy kotlin definitions from uniffi")
    print(f"destination: {dst}")

    for file in src_files:
        print(f"copy file: {file}")
        shutil.copy(file, dst)


if __name__ == "__main__":
    run()
