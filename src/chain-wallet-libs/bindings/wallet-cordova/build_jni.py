#!/usr/bin/env python3

from pathlib import Path
import subprocess
import sys
import shutil
from copy_jni_definitions import run as copy_definitions

libname = "libwallet_jni.so"
copy_to = Path("src/android/libs")
script_directory = Path(__file__).parent
root_directory = script_directory.parent.parent / "target"

targets = {
    "aarch64-linux-android": "arm64-v8a",
    "armv7-linux-androideabi": "armeabi-v7a",
    "i686-linux-android": "x86",
    "x86_64-linux-android": "x86_64",
}


def run(release=True):
    for rust_target, android_target in targets.items():
        arguments = [
            "cross",
            "rustc",
            "--target",
            rust_target,
            "-p",
            "wallet-jni",
        ]

        if release:
            arguments = arguments + ["--release", "--", "-C", "lto"]

        out = subprocess.run(arguments)

        if out.returncode != 0:
            print("couldn't build for target: ", rust_target)
            sys.exit(1)

        dst = script_directory / copy_to / android_target
        dst.mkdir(parents=True, exist_ok=True)

        src = root_directory / rust_target / "release" / libname
        shutil.copy(src, dst)

    copy_definitions()


if __name__ == "__main__":
    run()
