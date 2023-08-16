<!-- markdownlint-disable no-inline-html -->
<h1 align="center">Catalyst Core</h1>

<p align="center">
    <h2 align="center">Core Catalyst Governance Engine and utilities</h2>
</p>

<p align="center">
 <a href="https://github.com/input-output-hk/catalyst-core/actions/workflows/rust.yml">
    <img src="https://github.com/input-output-hk/catalyst-core/actions/workflows/rust.yml/badge.svg" alt="Current CI Status." />
  </a>
   <a href="https://github.com/input-output-hk/catalyst-core#license">
    <img src="https://img.shields.io/badge/license-MIT%2FApache--2.0-blue"
    alt="Catalyst Core is released under either of Apache License, Version 2.0 or MIT license at your option.." />
  </a>
  <a href="https://github.com/input-output-hk/catalyst-core/blob/main/CODE_OF_CONDUCT.md">
    <img src="https://img.shields.io/badge/PRs-welcome-brightgreen.svg" alt="PRs welcome!" />
  </a>
   <a href='https://coveralls.io/github/input-output-hk/catalyst-core?branch=main'><img src='https://coveralls.io/repos/github/input-output-hk/catalyst-core/badge.svg?branch=main' alt='Coverage Status' />
</a>
  <a href="https://api.securityscorecards.dev/projects/github.com/input-output-hk/catalyst-core">
    <img src="https://api.securityscorecards.dev/projects/github.com/input-output-hk/catalyst-core/badge" alt="OpenSSF Scorecard" />
  </a>
</p>
<!-- markdownlint-enable no-inline-html -->

# Content

- [Content](#content)
  - [What's inside?](#whats-inside)
  - [Requirements](#requirements)
  - [Development](#development)
    - [Earthly](#earthly)
    - [Manual](#manual)
      - [Prerequisites](#prerequisites)
        - [Linux](#linux)
        - [macOS](#macos)
        - [Windows](#windows)
    - [Nix](#nix)
    - [Install Extra Packages/Tools](#install-extra-packagestools)
  - [Building Documentation](#building-documentation)
  - [Support](#support)
  - [License](#license)

## What's inside?

- [Audit Tooling](https://github.com/input-output-hk/catalyst-core/blob/main/src/audit/README.md)
- [Catalyst Event Data Service](https://github.com/input-output-hk/catalyst-core/blob/main/src/cat-data-service/README.md)
- [Catalyst Toolbox](https://github.com/input-output-hk/catalyst-core/blob/main/src/catalyst-toolbox/README.md)
- [Chain Libraries](https://github.com/input-output-hk/catalyst-core/tree/main/src/chain-libs)
- [Chain Wallet Libraries](https://github.com/input-output-hk/catalyst-core/blob/main/src/chain-wallet-libs/README.md)
- [Catalyst Event Database](https://github.com/input-output-hk/catalyst-core/blob/main/src/event-db/Readme.md)
- [Jormungandr Node](https://github.com/input-output-hk/catalyst-core/blob/main/src/jormungandr/README.md)
- [Jortestkit](https://github.com/input-output-hk/catalyst-core/tree/main/src/jortestkit)
- [Vit Servicing Station](https://github.com/input-output-hk/catalyst-core/blob/main/src/vit-servicing-station/README.md)
- [Vit Testing](https://github.com/input-output-hk/catalyst-core/blob/main/src/vit-testing/README.md)
- [Voting Tools](https://github.com/input-output-hk/catalyst-core/blob/main/src/voting-tools-rs/README.md)

## Requirements

- [Rust](https://www.rust-lang.org/tools/install)
- [Docker](https://docs.docker.com/get-docker/)
- [Earthly](https://earthly.dev/get-earthly)

## Development

### Earthly

> **Note**: This is the preferred development environment.

Make sure that docker is running and then run:

```sh
# This command can take a while to run.
earthly +local
```
 then run:

```sh
docker-compose -f local/docker-compose.yml up
```

You should now be able to use [Catalyst Core API V1](https://input-output-hk.github.io/catalyst-core/main/07_web_api/catalyst-core-api.html) locally:

Open a new terminal window and run:

```sh
curl --request GET \
  --url http://localhost:3030/api/v1/events \
  --header 'Accept: application/json'
```

you should get list of events:

```json
[
  {
    "id": 0,
    "name": "Catalyst Fund 0",
    "starts": "2020-05-22T00:00:00+00:00",
    "ends": "2020-06-24T00:00:00+00:00",
    "final": true
  },
  {
    "id": 1,
    "name": "Catalyst Fund 1",
    "starts": "2020-08-08T00:00:00+00:00",
    "ends": "2020-09-22T00:00:00+00:00",
    "final": true
  },
.
.
.
.
  {
    "id": 9,
    "name": "Catalyst Fund 9",
    "starts": "2022-06-02T00:00:00+00:00",
    "ends": "2022-10-11T00:00:00+00:00",
    "final": true
  }
]
```

### Manual

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

- [choco](https://chocolatey.org/)

```sh
choco install protoc openssl sqlite postgresql14
```

### Nix

> **⛔️ Note**: Nix packaging code is deprecated and will be removed in a future.

- Install [Nix](https://nixos.org/download.html)
- Start a nix development environment (from the repo root): `nix develop`

### Install Extra Packages/Tools

This only needs to be done once when the development environment is created.

- `cargo install cargo-binstall --locked`
- `cargo binstall --no-confirm cargo-make`
- `cargo make install-prereqs`

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
