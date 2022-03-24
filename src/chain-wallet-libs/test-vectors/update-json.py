#!/usr/bin/env python

import binascii
import subprocess
import json

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
