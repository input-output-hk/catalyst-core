# Getting started

The javascript documentation for this module can be generated with jsdoc by
running:

```bash
npm install
npm run doc
```

The generated files can be found at the `doc` directory, and can be read by
opening `index.html` with a web browser.

At the moment the best source for examples are the javascript
[tests](tests/src/main.js).

# Development

## Getting started

The [official cordova
documentation](https://cordova.apache.org/docs/en/11.x/guide/hybrid/plugins/index.html)
is the best place to start.

## Requirements

### General

As a baseline, Node.js and the cordova cli are required. Since the process of
running the tests involves creating an application. The documentation at
[installing-the-cordova-cli](https://cordova.apache.org/docs/en/11.x/guide/cli/index.html#installing-the-cordova-cli)
can be used as a guide. Check out also the [Android
documentation](https://cordova.apache.org/docs/en/11.x/guide/platforms/android/index.html)
and the [iOS
documentation](https://cordova.apache.org/docs/en/11.x/guide/platforms/ios/plugin.html)
for requirements specific to the platform you are going to be developing for.

Additionally, python3 is required to run the helper scripts.

`jcli` is required to generate the genesis file that it is used in
the test-vectors, installation instructions can be found in the [jormungandr's
repository](https://github.com/input-output-hk/jormungandr). It's recommended
that the `jcli` version is built with the same version of `chain-libs` that is
used to build the plugin (which can be found in the Cargo.lock file), although
it's not strictly necessary as long as the genesis binary encoding is
compatible.

### Android

- [cross](https://github.com/cross-rs/cross) is currently used for building the
native libraries for Android.
- [uniffi-bindgen](https://github.com/mozilla/uniffi-rs). The version must be the same one that is used in the `wallet-uniffi` crate. This can be found [here](../wallet-uniffi/Cargo.toml).

### iOS

The ios rust platforms:

- `rustup target add x86_64-apple-ios`
- `rustup target add aarch64-apple-ios`

[cbindgen](https://github.com/eqrion/cbindgen) is necessary for regenerating the
C header, which is then used from the [Objetive C code](src/ios/WalletPlugin.m) in this package. Since the
latest version is in source control, this is only needed if the core API
changes. The [regen_header.sh](../bindings/wallet-c/regen_header.sh) script can
be used to do this.

## Overview

The core of the plugin is written in rust, and ffi is used to bridge that to
either Objective-C or Kotlin, depending on the platform.

The [wallet.js](www/wallet.js) file has the top level Javascript api for the
plugin users, which is mostly a one-to-one mapping to the API of the
wallet-core rust crate.

The iOS part of the plugin is backed by the [wallet-c](../wallet-c/wallet.h)
package, while the Android support is provided via the
[wallet-uniffi](../wallet-uniffi/src/lib.udl) package. Both are also thin
wrappers over **wallet-core**.

## Build

build uniffi bindings, inside `catalyst-core/src/chain-wallet-libs/bindings/wallet-uniffi`
```
cargo b --release
./gen_bindings.sh
```

[build_jni.py](scripts/build_jni.py) in the `scripts` directory will compile the
Android native libraries, generate the Kotlin bindings, and copy those to this
package in the `src/android` directory.
```
RUSTFLAGS="-C embed-bitcode" python3 build_jni.py
```

[build_ios.py](scripts/build_ios.py) in the `scripts` directory will compile the
iOS native libraries, and copy those along the C header to this package.
```
RUSTFLAGS="-C embed-bitcode" python3 build_ios.py
```

`npm pack` can be used to make a distributable version of the plugin as an npm
package.

## Running the tests

The *tests* directory contains a Cordova plugin with [js
tests](tests/src/main.js), we use
[cordova-plugin-test-framework](https://github.com/apache/cordova-plugin-test-framework)
as a test harness.

The [test.py](scripts/test.py) script can be used to build
the plugin and setup the test harness. For example, the following command will

- create a cordova application at the `~/cdvtest/hello` directory. The cdvtest directory must not exist, as the script will not overwrite it.
- install the cordova-plugin-test-framework.
- build the native libraries for the android platform, and copy those to
  src/android/libs.
- build the wallet-uniffi kotlin bindings for the native library.
- install the plugin at this directory.
- install the plugin in the tests directory.
- run the test application if there is an emulator or device available.

```bash
python3 test.py --platform android -d ~/cdvtest --cargo-build --run android full
```

The `reload-plugin` and `reload-tests` commands can be used if only one of
those was modified, to avoid having to redo the whole process.
