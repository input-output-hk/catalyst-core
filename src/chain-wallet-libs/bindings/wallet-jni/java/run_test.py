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


def compile():
    packageFiles = map(lambda path: str(path), (scriptdirectory /
                                                Path("com/iohk/jormungandrwallet")).glob("*.java"))
    out = subprocess.run([
        "javac", "-cp", classpath, "WalletTest.java"] + list(packageFiles), cwd=scriptdirectory)

    if out.returncode != 0:
        print("couldn't compile java files")
        print(f"command: {' '.join(out.args) }")
        sys.exit(1)


def run():
    compile()
    out = subprocess.run([
        "java", f"-Djava.library.path={dynlibdirectory}", "-cp", classpath, "org.junit.runner.JUnitCore", "WalletTest"
    ], cwd=scriptdirectory)

    if out.returncode != 0:
        print("couldn't run java files")
        print(f"command: {' '.join(out.args) }")
        sys.exit(1)


if __name__ == "__main__":
    run()
