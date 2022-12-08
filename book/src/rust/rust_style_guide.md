<h1 align="center">🦀 Rust Style Guide</h1>

- [Dependencies](#dependencies)
- [Patterns to follow](#patterns-to-follow)
- [Style Guide](#style-guide)
  - [The Cargo.toml File](#the-cargotoml-file)
  - [Feature Selection](#feature-selection)
  - [Imports and Exports](#imports-and-exports)

### Dependencies

Install clippy and rustfmt

```sh
rustup component add clippy
rustup component add rustfmt
```

### Patterns to follow

- For all new code, apply `clippy::pedantic` and `fmt` before creating a PR.

```sh
cargo clippy -- -W clippy::pedantic
cargo fmt
```

- Document all new public API interfaces.

### Style Guide

We are adopting the following style guide to keep code and documentation style consistent across all the code in the repository.
We begin with the formatting style enforced by the `stable` version of `rustfmt`with the configuration specified in the `.rustfmt.toml` file.
Beyond what rustfmt currently enforces, we have specified other rules below.


#### The Cargo.toml File

The Cargo.toml file should adhere to the following template:

```toml
[package]
name = "..."
version = "..."
edition = "..."
authors = ["IOG <contact@iohk.io>"]
readme = "README.md"
license-file = "LICENSE"
repository = "https://github.com/input-output-hk/repo-name"
homepage = "https://github.com/input-output-hk/repo-name"
documentation = "https://github.com/input-output-hk/repo-name"
categories = ["..."]
keywords = ["..."]
description = "..."
publish = false

[package.metadata.docs.rs]
# To build locally:
# RUSTDOCFLAGS="--cfg doc_cfg" cargo +nightly doc --all-features --open
all-features = true
rustdoc-args = ["--cfg", "doc_cfg"]

[badges]
is-it-maintained-issue-resolution = { repository = "input-output-hk/repo-rs" }
is-it-maintained-open-issues = { repository = "input-output-hk/repo-rs" }
maintenance = { status = "actively-developed" }

[[bin]]
...

[features]
...

[dependencies]
...

[dev-dependencies]
...

[build-dependencies]
...

[profile....]
...

```

Specifically, we have:

1. Use double quotes instead of single quotes.
2. Use the above as the standard ordering of the [package] map.
3. [[bin]] before [features] before [dependencies] before [dev-dependencies] before [build-dependencies] before [profile] settings.
4. Order features and dependencies alphabetically.
5. When selecting features for a [features] entry or the features on a dependency, order the features alphabetically.
6. For a given dependency, use the following structure with optional and features keys as needed:

```toml
crate-name = { version = "...", optional = true, default-features = false, features = ["..."] }
If the crate is a path or git dependency, replace those keys with the version key and add a tag, branch, or rev as needed following the git key.
```
7. When adding a feature, add a doc string in the title case and a new line between each feature.

#### Feature Selection

When using features, be sure to attach a doc_cfg feature declaration as well unless the code is not exported to pub.

```rust
#[cfg(feature = "...")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "...")))]
pub mod feature_gated_public_module;
```

#### Imports and Exports

TODO









