#!/usr/bin/env python3

from pathlib import Path
import subprocess
import shutil
import sys
import os

from directories import rust_build_directory, plugin_directory

libname = "libuniffi_jormungandr_wallet.so"
android_libs_directory = Path("src/android/libs")


# Keep all four targets
targets = {
    "aarch64-linux-android": "aarch64-linux-android",
    "armv7-linux-androideabi": "armv7a-linux-androideabi",
    "i686-linux-android": "i686-linux-android",
    "x86_64-linux-android": "x86_64-linux-android",
}

# Set API level and page size
ANDROID_API = "21"
PAGE_SIZE = "16384"

NDK_HOME = os.environ.get("ANDROID_NDK_HOME")
if not NDK_HOME:
    print("ERROR: ANDROID_NDK_HOME is not set")
    sys.exit(1)


def copy_libs(release=True):
    debug_or_release = "release" if release else "debug"

    for rust_target, android_target in targets.items():
        dst = plugin_directory / android_libs_directory / android_target
        dst.mkdir(parents=True, exist_ok=True)

        src = rust_build_directory / rust_target / debug_or_release / libname
        print(dst)
        shutil.copy(src, dst)


def run(release=True):
    for rust_target, android_target in targets.items():
        cargo_args = [
            "cargo", "rustc",
            "--target", rust_target,
            "-p", "wallet-uniffi",
            "--features", "builtin-bindgen",
        ]

        if release:
            cargo_args.append("--release")

        cargo_args.extend([
            "--",
            # "-C", f"link-arg=--target={rust_target}",
            "-C", f"link-arg=-Wl,-z,max-page-size={PAGE_SIZE}",
            "-C", f"linker={NDK_HOME}/toolchains/llvm/prebuilt/linux-x86_64/bin/{android_target}{ANDROID_API}-clang"
        ])

        print(f"Building target {rust_target}...")
        out = subprocess.run(cargo_args)
        if out.returncode != 0:
            print("Couldn't build target:", rust_target)
            sys.exit(1)

    copy_libs(release)


if __name__ == "__main__":
    run()
