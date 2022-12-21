<h1 align="center">🦀 Rust Style Guide</h1>

- [Dependencies](#dependencies)
- [Patterns to follow](#patterns-to-follow)
- [Style Guide](#style-guide)
  - [The Cargo.toml File](#the-cargotoml-file)
  - [Feature Selection](#feature-selection)
  - [Imports and Exports](#imports-and-exports)
  - [Traits](#traits)
    - [Defining Traits](#defining-traits)
    - [Implementing Traits](#implementing-traits)
  - [Crate lib.rs Module](#crate-librs-module)
  - [Ignoring Compiler Warnings](#ignoring-compiler-warnings)
  - [Where Clauses](#where-clauses)
  - [Magic Numbers](#magic-numbers)

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

- We encourage [documentation tests](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html) when it makes sense.

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

Imports (use) and exports (mod) should be ordered as follows:

1. External Crate Declarations

2. Private Imports

3. Private Imports with Features

4. Private Exports

5. Private Exports with Features

6. Public Exports

7. Public Exports with Features

8. Reexports

9. Reexports with Features

Here's an example set of declarations:

```rust
extern crate crate_name;

use module::submodule::entry;

#[cfg(feature = "...")]
use module::feature_gated_submodule;

mod another_module;
mod module;
mod the_third_module;

#[cfg(feature = "...")]
mod feature_gated_module;

pub mod public_module;

#[cfg(feature = "...")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "...")))]
pub mod feature_gated_public_module;

pub use reexported_objects;

#[cfg(feature = "...")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "...")))]
pub use feature_gated_reexported_objects;
```

Ensure that there are new lines between each category. Be sure that if some imports or exports are feature-gated, that they are sorted by feature alphabetically. If there is a feature gated import that requires importing multiple objects, use the following pattern:

```rust
#[cfg(feature = "...")]
use {
    thing1, thing2, thing3, thing4,
};
```

**NOTE**: All imports should occur at the top of any module, and a newline should be added between the last import and the first declared object.

#### Traits

##### Defining Traits

When defining a trait, use the following syntax:

```rust
/// DOCS
trait Trait<T> {
    /// DOCS
    type Type1: Default;

    /// DOCS
    type Type2;

    /// DOCS
    const CONST_1: usize;

    /// DOCS
    const CONST_2: usize;

    /// DOCS
    fn required_method(&self, argument: Self::Type1) -> T;

    /// DOCS
    #[inline]
    fn optional_method(&self) -> T {
        Self::required_method(Self::Type1::default())
    }
}
```

Notice the ordering of components:

1. Associated Types

2. Associated Constants

3. Methods

Depending on the context and presentation, you can mix the ordering of required and optional methods. Also, notice the name formatting, although clippy should detect if naming differs from this pattern.

##### Implementing Traits

When implementing traits, use the following syntax:

```rust
impl<T> Trait for Struct<T> {
    type Type1 = B;
    type Type2 = C;

    const CONST_1: usize = 3;
    const CONST_2: usize = 4;

    #[inline]
    fn required_method(&self, argument: Self::Type1) -> T {
        self.struct_method(argument).clone()
    }

    #[inline]
    fn optional_method(&self) -> T {
        short_cut_optimization(self)
    }
}
```

Notice the lack of space between implementations of the same category except for methods which always get a new line between them (like all methods). Only add space between types and constants if a comment is necessary, like in this example:

```rust
impl Configuration {
    const SPECIAL_CONSTANT: usize = 12345;

    /// In this case we have to use this constant because it's very special.
    const ANOTHER_SPECIAL_CONSTANT: usize = 9876;
}
```

Otherwise, it should look like:

```rust
impl Configuration {
    const SPECIAL_CONSTANT: usize = 12345;
    const ANOTHER_SPECIAL_CONSTANT: usize = 9876;
}
```

#### Crate lib.rs Module

Every crate has at least a lib.rs (or a main.rs if there is no library), and it should include at least these macro invocations:

```rust
//! Module Documentation

#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![forbid(rustdoc::broken_intra_doc_links)]
#![forbid(missing_docs)]

// IMPORTS AND EXPORTS GO HERE
```

#### Ignoring Compiler Warnings

Sometimes, we may want to ignore a particular compiler warning or clippy warning. This is especially true because of some false-positive error or because we are writing some generic macro code.
In either case, we need to mark the `#[allow(...)]` clause with a note on why we want to ignore this warning.

```rust
#[allow(clippy::some_lint)] // NOTE: Here's the reason why this is ok.
fn some_function() {}
```

In the case of allow we want to be careful of its scope to not ignore warnings except in the exact place where the unexpected behavior exists. Therefore, #[allow(...)] should be marked on functions and not modules, even if that means it is repeated multiple times. In some rare cases where this repetition would be too cumbersome and adding it to the module is cleaner, then also be sure to state in a note why this is better than marking it on the functions themselves.

**Known exceptions**

- `clippy::match_bool` - warns when writing match some_bool. However, this is often more concise than the alternative using if/else

- `clippy::module_name_repetition` - warns when creating an item whose name ends in the name of the module

- `clippy::derive_partial_eq_without_eq` - warns if you derive `PartialEq` and not `Eq`. This creates a semver hazard. Deriving Eq for types in a public API is a guarantee that they will continue to implement Eq until the next breaking change. This means that adding an f64 field, or any other type that doesn’t implement Eq is now a breaking change, which is a stronger commitment than should be default IMO.

- `clippy::missing_panics_doc` - warns if your function might panic, and you haven’t included a “panics” section in the doc comment. However, it doesn’t work all the time, and IMO leads to a false sense of security. Absence of a warning seems to imply absence from panics, but it doesn’t. This isn’t true for the related missing_error_docs and missing_safety_docs , so those should remain enabled. If the API legitimately is intended to panic under some conditions, you should still add a panic section to the docs (e.g. maybe some cryptography library requires that a certain input number is prime, the docs should say panics if foo is not prime). The reason this lint is disabled is because the implementation is buggy, not because we don’t want panic docs. This also doesn’t apply to panics that would be considered programming errors if they ever happened

#### Where Clauses
Always use where clauses instead of inline trait constraints. So instead of


```rust
fn function<T: Clone>(t: &T) -> T {
    t.clone()
}
```
you should use:

```rust
fn function<T>(t: &T) -> T
where
    T: Clone,
{
    t.clone()
}
```
This is true for any part of the code where generic types can be declared, like in fn, struct, enum, trait, and impl. The only "exception" is for supertraits, so use:

```rust
trait Trait: Clone + Default + Sized {}
```

Instead of using:

```rust
trait Trait
where
    Self: Clone + Default + Sized,
{}
```
Order where clause entries by declaration order, then by associated types and then by other constraints. Here's an example:

```rust
fn function<A, B, C>(a: &A, b: &mut B) -> Option<C>
where
    A: Clone + Iterator,
    B: Default + Eq,
    C: IntoIterator,
    A::Item: Clone,
    C::IntoIter: ExactSizeIterator,
    Object<B, C>: Copy,
```
**NOTE**: This rule is not so strict, and these where clauses should be organized in a way that makes the most sense but must follow this general rule.

Order each entries constraints alphabetically. Here's an example:

```rust
F: 'a + Copy + Trait + FnOnce(T) -> S
```

The ordering should be lifetimes first, then regular traits, then the function traits.

#### Magic Numbers
In general, we should avoid magic numbers and constants, but when necessary, they should be declared as such in some modules instead of being used in-line with no explanation. Instead of:

```rust
/// Checks that all the contributions in the round were valid.
pub fn check_all_contributions() -> Result<(), ContributionError> {
    for x in 0..7 {
        check_contribution(x)?;
    }
    Ok(())
}
```
you should use:

```rust
/// Contribution Count for the Some Protocol
pub const PARTICIPANT_COUNT: usize = 7;

/// Checks that all the contributions in the round were valid.
pub fn check_all_contributions() -> Result<(), ContributionError> {
    for x in 0..PARTICIPANT_COUNT {
        check_contribution(x)?;
    }
    Ok(())
}
```
Avoid situations where an arbitrary number needs to be chosen, and if so, prefer empirically measured numbers. If, for some reason, an arbitrary number needs to be chosen, and it should have a known order of magnitude, choose a power of two for the arbitrary number or something close to a power of two unless the situation calls for something distinctly not a power of two.







