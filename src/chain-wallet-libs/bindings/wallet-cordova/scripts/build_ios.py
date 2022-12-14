#!/usr/bin/env python3

from pathlib import Path
import subprocess
import sys
import shutil
from directories import (
    repository_directory,
    script_directory,
    rust_build_directory,
    plugin_directory,
)

libname = "libjormungandrwallet.a"

library_header_src = repository_directory / Path("bindings/wallet-c/wallet.h")
library_header_dst = plugin_directory / Path("src/ios/LibWallet.h")

targets = {
    "x86_64-apple-ios": "x86_64",
    "aarch64-apple-ios": "arm64",
}


def run(release=True):
    lipo_args = [
        "lipo",
        "-create",
        "-output",
        str(plugin_directory / "src/ios/" / libname),
    ]

    for rust_target, apple_target in targets.items():
        arguments = [
            "cargo",
            "rustc",
            "--target",
            rust_target,
            "-p",
            "jormungandrwallet",
        ]

        if release:
            arguments = arguments + ["--release", "--", "-C", "lto"]

        out = subprocess.run(arguments)
        if out.returncode != 0:
            print("couldn't build for target: ", rust_target)
            sys.exit(1)

        debug_or_release = "release" if release else "debug"

        lipo_args += [
            "-arch",
            apple_target,
            str(rust_build_directory / rust_target / debug_or_release / libname),
        ]

    out = subprocess.run(lipo_args)
    if out.returncode != 0:
        print("couldn't build universal lib")
        sys.exit(1)

    shutil.copy(library_header_src, library_header_dst)


if __name__ == "__main__":
    run()
