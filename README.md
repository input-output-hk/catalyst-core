<h1 align="center">Catalyst Core</h1>

<p align="center">
    <h2 align="center">Core Catalyst Governance Engine and utilities</h2>
</p>

# Content

- [Content](#content)
  - [What's inside?](#whats-inside)
  - [Requirements](#requirements)
  - [Development](#development)
  - [Prerequisites](#prerequisites)
    - [Development Environment (NIX Shell)](#development-environment-nix-shell)
    - [Development Environment (Manual)](#development-environment-manual)
    - [Install Extra Packages/Tools](#install-extra-packagestools)
  - [Testing](#testing)
  - [Deployment](#deployment)
  - [Building Documentation](#building-documentation)
  - [Support](#support)
  - [License](#license)

## What's inside?

TODO:

## Requirements

TODO:

## Development

## Prerequisites

### Development Environment (NIX Shell)

This is the **preferred** development environment.

1. Install NIX : <https://nixos.org/download.html>
2. Start a nix development environment (From the repo Root):
   `nix develop`

### Development Environment (Manual)

~~ ***NOT RECOMMENDED*** ~~

TODO...

### Install Extra Packages/Tools

This only needs to be done once when the development environment is created.

1. `cargo install cargo-make` : see <https://github.com/sagiegurari/cargo-make>
2. `cargo make install-prereqs`

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
