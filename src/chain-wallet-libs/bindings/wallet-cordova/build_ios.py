#!/usr/bin/env python3

from pathlib import Path
import subprocess
import sys
import shutil

libname = "libjormungandrwallet.a"
root_directory = Path("../../target/")

library_header_src = Path("../wallet-c/wallet.h")
library_header_dst = Path("src/ios/LibWallet.h")

targets = {
    "x86_64-apple-ios": "x86_64",
    "aarch64-apple-ios": "arm64",
}


def run(release=True):
    lipo_args = ["lipo", "-create", "-output", "./src/ios/" + libname]

    for rust_target, apple_target in targets.items():
        arguments = [
            "cross",
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
        lipo_args += [
            "-arch",
            apple_target,
            str(root_directory / rust_target / "release" / libname),
        ]

    out = subprocess.run(lipo_args)
    if out.returncode != 0:
        print("couldn't build universal lib")
        sys.exit(1)

    shutil.copy(library_header_src, library_header_dst)


if __name__ == "__main__":
    run()
