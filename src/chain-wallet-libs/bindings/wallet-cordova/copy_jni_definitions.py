#!/usr/bin/env python3

from pathlib import Path
import subprocess
import sys
import shutil

script_directory = Path(__file__).parent

def run():
    # copy java definitions from jni directory

    package_path = Path("com/iohk/jormungandrwallet")

    src_files = (script_directory.parent / "wallet-jni" /
                 "java" / "com" / "iohk" / "jormungandrwallet").glob("*java")

    dst = script_directory / Path("src/android/jormungandrwallet")
    dst.mkdir(parents=True, exist_ok=True)

    print("Copy java definitions from jni directory")
    print(f"destination: {dst}")

    for file in src_files:
        print(f"copy file: {file}")
        shutil.copy(file, dst)


if __name__ == "__main__":
    run()
