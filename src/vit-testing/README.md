# vit-testing

Incubator for catalyst related testing projects

User guide documentation available [here][docs]

[docs]: https://input-output-hk.github.io/vit-testing

## Master current build status

| CI | Status | Description |
|---:|:------:|:------------|
| CircleCI | [![CircleCI](https://circleci.com/gh/input-output-hk/vit-testing/tree/master.svg?style=svg)](https://circleci.com/gh/input-output-hk/vit-testings/tree/master) | Master and PRs |

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
* C compiler (see [cc-rs](https://github.com/alexcrichton/cc-rs) for more details):
  * Must be available as `cc` on Unix and MinGW.
  * Or as `cl.exe` on Windows.

#### Path

* Win: Add `%USERPROFILE%\.cargo\bin` to the  environment variable `PATH`.
* Lin/Mac: Add `${HOME}/.cargo/bin` to your `PATH`.

### Commands

```sh
git clone https://github.com/input-output-hk/vit-testing
cd vit-testing
cargo install --locked --path vitup
cargo install --locked --path valgrind
cargo install --locked --path iapyx
```

This will install 3 tools:

* `iapyx`: catalyst voting cli;
* `vitup`: a command line tool to help you bootstrap catalyst backend;
* `valgrind`: minimal proxy service for local deployment;

## Documentation

Documentation is available in the markdown format [here](doc/SUMMARY.md)

## License

This project is licensed under either of the following licenses:

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or
  <http://opensource.org/licenses/MIT>)
