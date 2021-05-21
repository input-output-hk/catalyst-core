#!/usr/bin/env python3

from pathlib import Path
import subprocess
import sys
import shutil
from copy_jni_definitions import run as copy_definitions
from directories import rust_build_directory, plugin_directory

libname = "libwallet_jni.so"
android_libs_directory = Path("src/android/libs")

targets = {
    "aarch64-linux-android": "arm64-v8a",
    "armv7-linux-androideabi": "armeabi-v7a",
    "i686-linux-android": "x86",
    "x86_64-linux-android": "x86_64",
}


def copy_libs(release=True):
    for rust_target, android_target in targets.items():
        dst = plugin_directory / android_libs_directory / android_target
        dst.mkdir(parents=True, exist_ok=True)

        debug_or_release = "release" if release else "debug"

        src = rust_build_directory / rust_target / debug_or_release / libname
        shutil.copy(src, dst)


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

    copy_libs(release)
    copy_definitions()


if __name__ == "__main__":
    run()
