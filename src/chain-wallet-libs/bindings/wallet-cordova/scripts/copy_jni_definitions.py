#!/usr/bin/env python3

from pathlib import Path
import subprocess
import sys
import shutil
from directories import repository_directory, plugin_directory


def run():
    # copy java definitions from jni directory

    package_path = Path("com/iohk/jormungandrwallet")

    src_files = (
        repository_directory
        / "bindings"
        / "wallet-jni"
        / "java"
        / "com"
        / "iohk"
        / "jormungandrwallet"
    ).glob("*java")

    dst = plugin_directory / Path("src/android/jormungandrwallet")
    dst.mkdir(parents=True, exist_ok=True)

    print("Copy java definitions from jni directory")
    print(f"destination: {dst}")

    for file in src_files:
        print(f"copy file: {file}")
        shutil.copy(file, dst)


if __name__ == "__main__":
    run()
