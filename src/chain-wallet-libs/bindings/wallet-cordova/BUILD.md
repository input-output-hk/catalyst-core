# Building the plugin

Before packaging the plugin, it's necessary to build some native libraries from the rust crates in this repo, and copy the files inside the plugin's directory.

## Android

For android, run `build_jni.py` in the plugin's root directory. This assumes a proper configuration to cross-compile rust to the android targets. Besides this, as a requirement to build the `clear_on_drop` dependency, a working C compiler for the target platform is needed. This can be configured, with, for example:

```
export CC_aarch64_linux_android=$NDK/toolchains/llvm/prebuilt/$BUILDING_PLATFORM/bin/aarch64-linux-android28-clang

export CC_armv7_linux_androideabi=$NDK/toolchains/llvm/prebuilt/$BUILDING_PLATFORM/bin/armv7a-linux-androideabi28-clang

export CC_i686_linux_android=$NDK/toolchains/llvm/prebuilt/$BUILDING_PLATFORM/bin/i686-linux-android28-clang

export CC_x86_64_linux_android=$NDK/toolchains/llvm/prebuilt/$BUILDING_PLATFORM/bin/x86_64-linux-android28-clang
```

Where $NDK is your * Android NDK * installation directory, and $BUILDING_PLATFORM is your current platform (not the target).

## iOS

Prerequisites:

* XCode
* `rustup target add x86_64-apple-ios` - for testing.
* `rustup target add aarch64-apple-ios` - for devices.

Run `./build_ios.py`. This will build the library and copy headers and the
library file to the Cordova plugin directory.

## Electron

Prerequisites:

* [wasm-pack](https://github.com/rustwasm/wasm-pack)

run `build_wasm.py` in the plugin's root directory.
