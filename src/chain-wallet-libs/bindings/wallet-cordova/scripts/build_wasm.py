#!/usr/bin/env python3

from pathlib import Path
import subprocess
import sys
import shutil
from directories import repository_directory, plugin_directory


def run():
    wallet_js = repository_directory / "bindings" / "wallet-js"
    relative_path_from_wallet_js = Path("../wallet-cordova/src/electron/pkg")
    out = subprocess.run(
        [
            "wasm-pack",
            "build",
            "--target",
            "no-modules",
            "-d",
            relative_path_from_wallet_js,
            wallet_js,
        ]
    )

    if out.returncode != 0:
        print("couldn't build js bindings ")
        sys.exit(1)

    # the output of 'no-modules' target generates a umd-style module, this means it adds a 'wasm_bindgen' variable to the global namespace
    # when we source the file by using the <js-module> cordova plugin directive, it is automagically wrapped with a cordova amd-style module
    # we append the custom export of the 'global' (but it is not global, because of the wrap) in order to be able to use `cordova.require`
    with open(
        plugin_directory / Path("src/electron/pkg/wallet_js.js"), "a"
    ) as file_object:
        file_object.write("\nmodule.exports = wasm_bindgen")


if __name__ == "__main__":
    run()
