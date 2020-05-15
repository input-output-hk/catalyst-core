#!/usr/bin/env python3

from pathlib import Path
import subprocess
import sys
import shutil

libname = "libjormungandrwallet.a"
copy_to = Path("src/ios");
root_directory = Path("../../target/")

library_header_src = Path("../wallet-c/wallet.h")
library_header_dst = Path("src/ios/LibWallet.h")

def run():
    out = subprocess.run(["cargo", "lipo", "--release", "-p", "jormungandrwallet"])

    if out.returncode != 0:
        print("couldn't build for target: ", rust_target)
        sys.exit(1)

    src = root_directory / "universal" / "release" / libname
    shutil.copy(src, copy_to)

    shutil.copy(library_header_src, library_header_dst)


if __name__ == "__main__":
    run()
