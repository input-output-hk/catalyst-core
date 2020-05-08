#!/usr/bin/env python3

from pathlib import Path
import subprocess
import sys
import shutil

libname = "libwallet_jni.so"
copy_to = Path("src/android/libs")
script_directory = Path(__file__).parent
root_directory = script_directory.parent.parent / "target"

targets = {
    "aarch64-linux-android": "amd64-v8a",
    "armv7-linux-androideabi": "armeabi-v7a",
    "i686-linux-android": "x86",
    "x86_64-linux-android": "x86_64",
}


def run():
    for rust_target, android_target in targets.items():
        out = subprocess.run(["cargo", "rustc", "--release", "--target",
                              rust_target, "-p" "wallet-jni", "--", "-C", "lto"])

        if out.returncode != 0:
            print("couldn't build for target: ", rust_target)
            sys.exit(1)

        dst = (script_directory / copy_to / android_target)
        dst.mkdir(parents=True, exist_ok=True)

        src = root_directory / rust_target / "release" / libname
        shutil.copy(src, dst)

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
