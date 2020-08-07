#!/usr/bin/env python3

from pathlib import Path
import subprocess
import sys

scriptdirectory = Path(__file__).parent
rootdirectory = scriptdirectory.parent.parent.parent
dynlibdirectory = rootdirectory / "target" / "debug"

junit = "junit-4.13.jar"
hamcrest = "hamcrest-core-1.3.jar"

classpath = f".:{junit}:{hamcrest}"
javafilesdir = Path("com/iohk/jormungandrwallet")


def compile_java_classes():
    packageFiles = map(str, (scriptdirectory /
                             Path("com/iohk/jormungandrwallet")).glob("*.java"))
    out = subprocess.run([
        "javac", "-cp", classpath, "WalletTest.java", *packageFiles], cwd=scriptdirectory)

    if out.returncode != 0:
        print("couldn't compile java files")
        print(f"command: {' '.join(out.args) }")
        sys.exit(1)


def compile_jni():
    build_jni = subprocess.run(
        ["cargo", "build", "-p" "wallet-jni"])

    if build_jni.returncode != 0:
        print(f"failed to build jni, command:\n {' '.join(build_jni.args) }")
        sys.exit(1)


def run():
    compile_java_classes()
    compile_jni()

    out = subprocess.run([
        "java", f"-Djava.library.path={dynlibdirectory}", "-cp", classpath, "org.junit.runner.JUnitCore", "WalletTest"
    ], cwd=scriptdirectory)

    if out.returncode != 0:
        print(f"command: {' '.join(out.args) }")
        sys.exit(1)


if __name__ == "__main__":
    run()
