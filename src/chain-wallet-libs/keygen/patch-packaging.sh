#!/bin/sh -e

# package.json needs to be fixed up due to a known issue in wasm-pack:
# https://github.com/rustwasm/wasm-pack/issues/837

cat pkg/package.json | jq '.files |= (.+ ["keygen_bg.js"] | unique)' > tmp.json
mv tmp.json pkg/package.json
