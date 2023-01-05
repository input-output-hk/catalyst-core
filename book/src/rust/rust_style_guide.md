# Rust Style Guide

This guide is intended to be a set of guidelines, not hard rules.
These represent the "default" for Rust code.
Exceptions can (and sometimes should) be made, but they should have a comment explaining what is special about this code that means the rule shouldn't apply here.

## Toolchain

We use the latest stable version of Rust.
You can get an up-to-date toolchain by running `nix develop`.
If you're not a Nix user, make sure you have the correct versions.


## Basic Rules

 - Formatting is "whatever `rustfmt` does". In cases where `rustfmt` doesn't yet work (i.e. macros, `let-else`), try to stay consistent with the rest of the codebase
 - Clippy should be used whenever possible, with `pedantic` lints turned on. There are some lints (particularly those from `pedantic`) that are generally unhelpful, often due to high false positive rates. There is a list of known exceptions that can be added to if you run into anything particularly bad
 - Clippy is not enabled for older parts of the codebase. This is allowed for legacy code, but any new code should have clippy enabled. We're working to get it enabled on everything
 - Avoid raw identifiers. Instead, use abbreviations/misspellings (i.e. `r#crate` -> `krate`, `r#type` -> `ty`, etc)

TLDR: run:
```text
cargo fmt
cargo clippy
cargo clippy --all-features
```
before submitting a PR

### Creating a new crate

We add the following preamble to all crates' `lib.rs`:
```rust
#[warn(clippy::pedantic)]
#[forbid(missing_docs)]
#[allow(/* known bad lints outlined below */)]
```

We enable `#[forbid(missing_docs)]` for a couple of reasons:
 - it forces developers to write doc comments for publicly exported items
 - it serves as a reminder that the item you're working on is part of your **public API**

### Exceptions for clippy

These lints are disabled:
 - `clippy::match_bool` - sometimes a `match` statement with `true => ` and `false => ` arms is sometimes more concise and equally readable
 - `clippy::module_name_repetition` - warns when creating an item with a name than ends with the name of the module it's in
 - `clippy::derive_partial_eq_without_eq` - warns when deriving `PartialEq` and not `Eq`. This is a semver hazard. Deriving `Eq` is a stronger semver guarantee than just `PartialEq`, and shouldn't be the default.
 - `clippy::missing_panics_doc` - this lint warns when a function might panic, but the docs don't have a `panics` section. This lint is buggy, and doesn't correctly identify all panics. You should still add panic docs if a function is **intended to panic** under some conditions. If a panic may occur, but you'd consider it a bug if it did, don't document it. We disable this lint because it creates a false sense of security.


## Guidelines

### Prefer references over generics

It's tempting to write a function like this:
```rust
fn use_str(s: impl AsRef<str>) {
  let s = s.as_ref();
  println!("{s}");
}
```


