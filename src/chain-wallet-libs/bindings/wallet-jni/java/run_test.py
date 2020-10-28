#!/usr/bin/env python3

from pathlib import Path
import subprocess
import sys
import argparse

scriptdirectory = Path(__file__).parent
rootdirectory = scriptdirectory.parent.parent.parent

junit = "junit-4.13.jar"
hamcrest = "hamcrest-core-1.3.jar"

classpath = f".:{junit}:{hamcrest}"
javafilesdir = Path("com/iohk/jormungandrwallet")


def compile_java_classes():
    packageFiles = map(str, (
        Path("com/iohk/jormungandrwallet")).glob("*.java"))

    out = subprocess.run([
        "javac", "-cp", classpath, "WalletTest.java", *packageFiles], cwd=scriptdirectory)

    if out.returncode != 0:
        print("couldn't compile java files")
        print(f"command: {' '.join(out.args) }")
        sys.exit(1)


def compile_jni(target):
    args = ["cargo", "build", "-p" "wallet-jni"]
    if target:
        args.append("--target")
        args.append(target)

    build_jni = subprocess.run(args, cwd=rootdirectory)

    if build_jni.returncode != 0:
        print(f"failed to build jni, command:\n {' '.join(build_jni.args) }")
        sys.exit(1)


def run():
    parser = argparse.ArgumentParser(description='run tests')
    parser.add_argument('--target', metavar='TARGET', type=str,
                        help='target to use with cargo build', default=None)

    args = parser.parse_args()

    compile_java_classes()
    compile_jni(args.target)

    dynlibdirectory = rootdirectory / "target" / \
        (args.target if args.target else ".") / "debug"

    out = subprocess.run([
        "java", f"-Djava.library.path={dynlibdirectory.resolve()}", "-cp", classpath, "org.junit.runner.JUnitCore", "WalletTest"
    ], cwd=scriptdirectory)

    if out.returncode != 0:
        print(f"command: {' '.join(out.args) }")
        sys.exit(1)


if __name__ == "__main__":
    run()
