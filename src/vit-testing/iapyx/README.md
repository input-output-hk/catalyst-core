# iapyx

Test wallet based on rust chain-wallet-libs.

## Install from Source

### Prerequisites

#### Rust

Get the [Rust Compiler](https://www.rust-lang.org/tools/install) (latest stable
version is recommended, minimum required: 1.39+).

```sh
rustup install stable
rustup default stable
rustc --version # if this fails, try a new command window, or add the path (see below)
```

#### Dependencies

* For detecting build dependencies:
  * Homebrew on macOS.
  * `vcpkg` on Windows.
  * `pkg-config` on other Unix-like systems.

#### Path

* Win: Add `%USERPROFILE%\.cargo\bin` to the  environment variable `PATH`.
* Lin/Mac: Add `${HOME}/.cargo/bin` to your `PATH`.

### Commands

```sh
git clone https://github.com/input-output-hk/vit-testing
cd vit-testing/iapyx
cargo install --locked --path . --force
```

This will install 3 tools:

* `iapyx-cli`: a command line tool to help you tests voting (NOTE: it is only a test tool it is not recommended to use it for any different purpose than testing);
* `iapyx-qr`: a command line tool to help you validate your qr code;
* `iapyx-load`: a command line tool use in load tests scenarios;

## iapyx-cli

command line tool to perform operations on wallet (recover,voting etc). Has almost full capability of voting like Voting app.

## iapyx-load

load tool for generating load over backend

example:
`cargo run --bin iapyx-load -- --address 127.0.0.1:8000 --pace 100 --progress-bar-mode monitor --threads 4 -- mnemonics .\mnemonics.txt`

Where mnemonics.txt should have format like:

```
town lift follow more chronic lunch weird uniform earth census proof cave gap fancy topic year leader phrase state circle cloth reward dish survey act punch bounce
neck bulb teach illegal try monitor claw rival amount boring provide village rival draft stone
```

## iapyx-qr

Utility tool for qr operations (validation etc.).

### Validate qr

In order to validate QR code you need to download it to an image fiel and crop in a way that qr code takes entire image space.

For example if we downloaded qr code as qr_code.png and our pin is 7777 we can check if is correct by running command:

```
iapyx-qr check-address --pin 7777 --qr qr_code.png
```

If Qr code is correct you should get output similar to:

```
Decoding qr from file: "qr_code.png"...
Address: ca1qdqpk32aepcrdsj5hl5vk0puhf884sj7wm3k3ytvphnjr3uuxnprtwpqr31
```

From the other side if pin is incorrect you should see output like this:

```
> iapyx-qr check-address --pin 1111 --qr qr_code.png

Decoding qr from file: "qr_code.png"...
thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: PinError(UnableToDecodeQr(SymmetricCipher(AuthenticationFailed)))', iapyx\src\bin\iapyx-qr.rs:5:40
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

If qr code is invalid you should expect

```
iapyx-qr check-address --pin 7777 --qr invalid_qr_code.png

Decoding qr from file: "C:\\Users\\Dariusz\\Desktop\\InvalidQRCode.png"...
thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: PinError(UnableToDecodeQr(HexDecodeError(InvalidHexCharacter { c: 'H', index: 0 })))', iapyx\src\bin\iapyx-qr.rs:5:40
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
