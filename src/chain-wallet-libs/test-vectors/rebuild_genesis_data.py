#!/usr/bin/env python

import binascii
import subprocess
import json
import os
from pathlib import Path

script_directory = Path(__file__).parent

os.chdir(script_directory)


def regen_block_bin():
    # make sure the block0 file is up-to-date
    # although this depends on the installed version of jcli
    subprocess.check_call(
        [
            "jcli",
            "genesis",
            "encode",
            "--input",
            "genesis.yaml",
            "--output",
            "block0",
        ],
    )


def update_json_file():
    # the json file has the genesis id and the block as hex, this makes it
    # easier to read it from the cordova js tests
    blockHex = None

    with open("block0", "rb") as file:
        binaryData = file.read()

        blockHex = binascii.hexlify(binaryData)

    blockId = subprocess.check_output(
        ["jcli", "genesis", "hash", "--input", "block0"],
    ).strip()

    with open("block0.json", "w") as file:
        file.write(
            json.dumps(
                {
                    "id": blockId.decode("utf-8"),
                    "hex": blockHex.decode("utf-8"),
                }
            )
        )


def run():
    regen_block_bin()
    update_json_file()


if __name__ == "__main__":
    run()
