#!/bin/sh -e

## FIXME: Technical debt, find the real issue in wasm-pack.
## For some reason this file is missing from the 'files' entry in package.json,
## causing the file not to be included in the build, which at least fails for
## the bundle target.

cat pkg/package.json | jq '.files |= (.+ ["wallet_bg.js"] | unique)' > tmp.json
mv tmp.json pkg/package.json
