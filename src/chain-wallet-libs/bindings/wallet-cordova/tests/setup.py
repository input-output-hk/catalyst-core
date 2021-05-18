#!/usr/bin/env python

# cordova create hello com.example.hello HelloWorld
# cd hello
# cordova platform add android
# cordova plugin add cordova-plugin-test-framework
# cordova plugin add this-plugin-path
# cordova plugin add path-to-wallet-cordova/tests
# sed 's/<content src="index.html" \/>/<content src="cdvtests\/index.html" \/>/' config.xml -i
# cordova build
# cordova run android

import subprocess

from pathlib import Path
import subprocess
import sys
import shutil
import argparse
import os
import re

sys.path.append(str(Path(__file__).parent.parent))
from build_jni import run as build_jni
from build_jni import copy_libs as copy_jni_libs
from copy_jni_definitions import run as copy_jni_definitions
from build_ios import run as build_ios

root_directory = Path("../../../")


def sed(original: str, replacement: str, file: Path):
    # TODO: this may have some problems, but I'm also not sure if I want to use
    # `sed`, mostly for Windows compatibility
    with open(file, "r") as config:
        lines = config.readlines()

    with open(file, "w") as config:
        for line in lines:
            config.write(re.sub(original, replacement, line))


def create_hello_world(build_dir: Path):
    os.makedirs(build_dir, exist_ok=True)

    subprocess.call(
        ["cordova", "create", "hello", "com.example.hello", "HelloWorld"],
        cwd=build_dir,
    )


def install_test_framework(app_dir: Path):
    subprocess.call(
        ["cordova", "plugin", "add", "cordova-plugin-test-framework"], cwd=app_dir
    )

    sed(
        '<content src="index.html" />',
        '<content src="cdvtests/index.html" />',
        app_dir / "config.xml",
    )


def install_platforms(app_dir: Path, android=True, ios=True):
    if android:
        subprocess.call(["cordova", "platform", "add", "android"], cwd=app_dir)

    if ios:
        subprocess.call(["cordova", "platform", "add", "ios"], cwd=app_dir)


def install_main_plugin(
    app_dir: Path, reinstall=True, android=False, ios=False, cargo_build=True
):
    plugin_path = Path(__file__).parent.parent

    print(f"plugin_path: {plugin_path}")

    if reinstall:
        subprocess.call(
            ["cordova", "plugin", "rm", "wallet-cordova-plugin"], cwd=app_dir
        )

    if android:
        if cargo_build:
            build_jni(release=False)
        else:
            copy_jni_libs(release=False)
            copy_jni_definitions()

    if ios and cargo_build:
        build_ios()

    subprocess.call(["cordova", "plugin", "add", str(plugin_path)], cwd=app_dir)


def install_test_plugin(app_dir: Path, reinstall=True):
    tests_path = Path(__file__).parent

    print(f"tests_path: {tests_path}")

    subprocess.call(["npm", "run", "build"], cwd=tests_path)

    if reinstall:
        subprocess.call(
            ["cordova", "plugin", "rm", "wallet-cordova-plugin-tests"], cwd=app_dir
        )

    subprocess.call(["cordova", "plugin", "add", str(tests_path)], cwd=app_dir)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Create test harness")

    platform_choices = ["android", "ios"]

    parser.add_argument(
        "--platform", required=True, nargs="+", choices=platform_choices
    )
    parser.add_argument("command", choices=["setup", "plugin", "tests"])
    parser.add_argument("-d", "--directory", type=Path)
    parser.add_argument("-r", "--run", choices=platform_choices)

    parser.add_argument("--cargo-build", dest="cargo_build", action="store_true")
    parser.add_argument("--no-cargo-build", dest="cargo_build", action="store_false")
    parser.set_defaults(feature=True)

    args = parser.parse_args()

    android = "android" in args.platform
    ios = "ios" in args.platform

    build_dir = args.directory

    app_dir = build_dir / "hello"

    if args.command == "setup":
        create_hello_world(build_dir)
        install_platforms(app_dir, android=android, ios=ios)
        install_test_framework(app_dir)
        install_main_plugin(
            app_dir,
            reinstall=False,
            android=android,
            ios=ios,
            cargo_build=args.cargo_build,
        )
        install_test_plugin(app_dir, reinstall=False)

    if args.command == "plugin":
        install_main_plugin(app_dir, reinstall=True, android=android, ios=ios)

    if args.command == "tests":
        install_test_plugin(app_dir, reinstall=True)

    subprocess.call(["cordova", "build"], cwd=app_dir)
    if args.run:
        subprocess.call(["cordova", "run", args.run], cwd=app_dir)
