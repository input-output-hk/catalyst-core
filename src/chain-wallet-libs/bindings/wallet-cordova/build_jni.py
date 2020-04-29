#!/usr/bin/env python3

from pathlib import Path
import subprocess
import sys
import shutil

libname = "libwallet_jni.so"
copy_to = Path("src/android/libs");
root_directory = Path("../../target/") 

targets = {
    "aarch64-linux-android" : "amd64-v8a",
    "armv7-linux-androideabi" : "armeabi-v7a",
    "i686-linux-android" : "x86",
    "x86_64-linux-android" : "x86_64",
}

def run():
    for rust_target, android_target in targets.items():
        out = subprocess.run(["cargo", "rustc", "--release", "--target", rust_target, "-p" "wallet-jni", "--", "-C", "lto"])

        if out.returncode != 0:
            print("couldn't build for target: ", rust_target)
            sys.exit(1)

        dst = (copy_to / android_target)
        dst.mkdir(parents=True, exist_ok=True);

        src = root_directory / rust_target / "release" / libname
        shutil.copy(src, dst)


if __name__ == "__main__":
    run()