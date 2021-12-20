#!/usr/bin/env python

import subprocess

from pathlib import Path
import subprocess
import sys
import shutil
import argparse
import os
import re

from build_jni import run as build_jni
from build_jni import copy_libs as copy_jni_libs
from copy_jni_definitions import run as copy_jni_definitions
from build_ios import run as build_ios
from directories import repository_directory, plugin_directory, tests_directory


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

    subprocess.check_call(
        ["cordova", "create", "hello", "com.example.hello", "HelloWorld"],
        cwd=build_dir,
    )


def enable_kotlin(app_dir: Path):
    file = app_dir / "config.xml"

    with open(file, "r") as config:
        lines = config.readlines()

    with open(file, "w") as config:
        for line in lines[:-1]:
            config.write(line)

        config.write('    <preference name="GradlePluginKotlinEnabled" value="true" />')
        config.write(
            '    <preference name="GradlePluginKotlinCodeStyle" value="official" />'
        )
        config.write(
            '    <preference name="GradlePluginKotlinVersion" value="1.3.50" />'
        )

        config.write(lines[-1])


def install_test_framework(app_dir: Path):
    subprocess.check_call(
        ["cordova", "plugin", "add", "cordova-plugin-test-framework"], cwd=app_dir
    )

    sed(
        '<content src="index.html" />',
        '<content src="cdvtests/index.html" />',
        app_dir / "config.xml",
    )


def install_platforms(app_dir: Path, android=True, ios=True):
    if android:
        subprocess.check_call(
            ["cordova", "platform", "add", "android@10.1.1"], cwd=app_dir
        )
        subprocess.check_call(["cordova", "requirements", "android"], cwd=app_dir)

    if ios:
        subprocess.check_call(["cordova", "platform", "add", "ios"], cwd=app_dir)
        subprocess.check_call(["cordova", "requirements", "ios"], cwd=app_dir)


def install_main_plugin(
    app_dir: Path, reinstall=True, android=False, ios=False, cargo_build=True
):
    if reinstall:
        subprocess.check_call(
            ["cordova", "plugin", "rm", "wallet-cordova-plugin"], cwd=app_dir
        )

    if android:
        if cargo_build:
            build_jni(release=False)
        else:
            copy_jni_libs(release=False)
            copy_jni_definitions()

    if ios:
        build_ios(release=False)

    subprocess.check_call(
        ["cordova", "plugin", "add", str(plugin_directory)], cwd=app_dir
    )


def install_test_plugin(app_dir: Path, reinstall=True):
    subprocess.check_call(["npm", "install"], cwd=tests_directory)
    subprocess.check_call(["npm", "run", "build"], cwd=tests_directory)

    if reinstall:
        subprocess.check_call(
            ["cordova", "plugin", "rm", "wallet-cordova-plugin-tests"], cwd=app_dir
        )

    subprocess.check_call(
        ["cordova", "plugin", "add", str(tests_directory)], cwd=app_dir
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Create test harness")

    platform_choices = ["android", "ios"]

    parser.add_argument(
        "--platform", required=True, nargs="+", choices=platform_choices
    )
    parser.add_argument("command", choices=["full", "reload-plugin", "reload-tests"])
    parser.add_argument("-d", "--directory", type=Path, required=True)
    parser.add_argument("-r", "--run", choices=platform_choices)

    parser.add_argument("--cargo-build", dest="cargo_build", action="store_true")
    parser.add_argument("--no-cargo-build", dest="cargo_build", action="store_false")
    parser.set_defaults(feature=True)

    args = parser.parse_args()

    android = "android" in args.platform
    ios = "ios" in args.platform

    build_dir = args.directory

    app_dir = build_dir / "hello"

    if args.command == "full":
        create_hello_world(build_dir)
        enable_kotlin(app_dir)
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

    if args.command == "reload-plugin":
        install_main_plugin(app_dir, reinstall=True, android=android, ios=ios)

    if args.command == "reload-tests":
        install_test_plugin(app_dir, reinstall=True)

    if ios:
        subprocess.check_call(
            [
                "cordova",
                "build",
                "ios",
                "--debug",
            ],
            cwd=app_dir,
        )

    if android:
        subprocess.check_call(["cordova", "build", "android"], cwd=app_dir)

    if args.run:
        subprocess.check_call(["cordova", "run", args.run], cwd=app_dir)
