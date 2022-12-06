<h1 align="center">Catalyst Core</h1>

<p align="center">
    <h2 align="center">Core Catalyst Governance Engine and utilities</h2>
</p>

<p align="center">
 <a href="https://github.com/input-output-hk/catalyst-core/actions/workflows/ci_tests.yml">
    <img src="https://github.com/input-output-hk/catalyst-core/actions/workflows/ci_tests.yml/badge.svg" alt="Current CI Status." />
  </a>
   <a href="https://github.com/input-output-hk/catalyst-core#license">
    <img src="https://img.shields.io/badge/license-MIT%2FApache--2.0-blue" alt="Catalyst Core is released under either of Apache License, Version 2.0 or MIT license at your option.." />
  </a>
  <a href="https://github.com/input-output-hk/catalyst-core/blob/main/CODE_OF_CONDUCT.md">
    <img src="https://img.shields.io/badge/PRs-welcome-brightgreen.svg" alt="PRs welcome!" />
  </a>
</p>

# Content

- [Content](#content)
  - [What's inside?](#whats-inside)
  - [Requirements](#requirements)
  - [Development](#development)
    - [Development Environment (NIX Shell)](#development-environment-nix-shell)
    - [Development Environment (Manual)](#development-environment-manual)
      - [Prerequisites](#prerequisites)
        - [Linux](#linux)
        - [macOS](#macos)
        - [Windows](#windows)
    - [Install Extra Packages/Tools](#install-extra-packagestools)
  - [Testing](#testing)
  - [Deployment](#deployment)
  - [Building Documentation](#building-documentation)
  - [Support](#support)
  - [License](#license)

## What's inside?

TODO:

## Requirements

## Development


### Development Environment (NIX Shell)

This is the **preferred** development environment.

1. Install NIX : <https://nixos.org/download.html>
2. Start a nix development environment (From the repo Root):
   `nix develop`

### Development Environment (Manual)

~~ ***NOT RECOMMENDED*** ~~

#### Prerequisites

##### Linux

```sh
sudo apt install -y protobuf-compiler libssl-dev libpq-dev libsqlite3-dev pkg-config
```

##### macOS

- [brew](https://brew.sh/)

```sh
brew install protobuf-c libsigsegv libpq libserdes pkg-config
```

##### Windows

```sh
TODO
```

### Install Extra Packages/Tools

This only needs to be done once when the development environment is created.

1. `cargo install cargo-binstall --locked` : see <https://github.com/cargo-bins/cargo-binstall>
2. `cargo binstall --no-confirm cargo-make` : see <https://github.com/sagiegurari/cargo-make>
3. `cargo make install-prereqs`

TODO: Can this (or an equivalent) be done by the devshell?

## Testing

TODO:

## Deployment

TODO:

## Building Documentation

If you have edited any of the documentation, then it needs to be updated by running:

```sh
cargo make build-docs
```

Any update files need to be committed to the repo. (until we have this integrated with CI).

## Support

Post issues and feature requests on the GitHub [issue tracker](https://github.com/input-output-hk/catalyst-core/issues).

## License

Licensed under either of [Apache License](LICENSE-APACHE), Version
2.0 or [MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
