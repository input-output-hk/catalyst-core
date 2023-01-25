# 🦀 Rust Style Guide

This guide is intended to be a set of guidelines, not hard rules.
These represent the *default* for Rust code.
Exceptions can (and sometimes should) be made, however:

* They should have a comment explaining what is special about this code which means the rule shouldn't apply here.
* Exceptions should be highlighted in the PR and discussed with the team.

* [🦀 Rust Style Guide](#-rust-style-guide)
  * [Toolchain](#toolchain)
  * [Basic Rules](#basic-rules)
    * [Creating a new crate](#creating-a-new-crate)
    * [Exceptions for clippy](#exceptions-for-clippy)
  * [Guidelines](#guidelines)
    * [Prefer references over generics](#prefer-references-over-generics)
    * [Abbreviations and naming things](#abbreviations-and-naming-things)
      * [General advice around names](#general-advice-around-names)
  * [Pay attention to the public API of your crate](#pay-attention-to-the-public-api-of-your-crate)
  * [Type safety](#type-safety)
    * [Use newtypes (a.k.a. microtypes)](#use-newtypes-aka-microtypes)
    * [Don't over-abstract](#dont-over-abstract)
  * [Unsafe code](#unsafe-code)
  * [Docs](#docs)
    * [Doctests](#doctests)
  * [Write code as if it's going to be in a web server](#write-code-as-if-its-going-to-be-in-a-web-server)
  * [Error handling](#error-handling)
    * [Handling expected errors](#handling-expected-errors)
      * [Use `thiserror` for recoverable errors](#use-thiserror-for-recoverable-errors)
      * [Use `color_eyre` for unrecoverable errors](#use-color_eyre-for-unrecoverable-errors)

## Toolchain

We use **the latest stable version** of Rust.
You can get an up-to-date toolchain by running `nix develop`.
If you're not a Nix user, make sure you have the correct versions.

## Basic Rules

* Formatting is "whatever `rustfmt` does".
  In cases where `rustfmt` doesn't yet work (i.e. macros, `let-else`), try to stay consistent with the rest of the codebase.
* Clippy should be used whenever possible, with `pedantic` lints turned on.
  There are some lints (particularly those from `pedantic`) that are generally unhelpful, often due to high false positive rates
  There is a list of known exceptions that can be added to if you run into anything particularly bad.
* Clippy is not enabled for older parts of the codebase.
  This is allowed for legacy code, but any new code should have `clippy` enabled.
  We're actively working to get it enabled on everything
* Avoid raw identifiers. Instead, use abbreviations/misspellings (i.e. `r#crate` -> `krate`, `r#type` -> `ty`, etc)

TLDR: run:

```text
cargo fmt
cargo clippy
cargo `clippy` --all-features
```

before submitting a PR

### Creating a new crate

We add the following preamble to all crates' `lib.rs`:

```rust
#![warn(clippy::pedantic)]
#![forbid(clippy::integer_arithmetic)]
#![forbid(missing_docs)]
#![forbid(unsafe_code)]
#![allow(/* known bad lints outlined below */)]
```

We enable `#![forbid(missing_docs)]` for a couple of reasons:

* it forces developers to write doc comments for publicly exported items
* it serves as a reminder that the item you're working on is part of your **public API**

We enable `#![forbid(unsafe_code)]` to reinforce the fact that **unsafe code should not be mixed in with the rest of our code**.
More details are below.

We enable `#![forbid(integer_arithmetic)]` to prevent you from writing code like:

```rust
let x = 1;
let y = 2;
let z = x + y;
```

Why is this bad?

Integer arithmetic may panic or behave unexpectedly depending on build settings.
In debug mode, overflows cause a panic, but in release mode, they silently wrap.
In both modes, division by `0` causes a panic.

By forbidding integer arithmetic, you have to choose a behaviour, by writing either:

* `a.checked_add(b)` to return an `Option` that you can error-handle
* `a.saturating_add(b)` to return a + b, or the max value if an overflow occurred
* `a.wrapping_add(b)` to return a + b, wrapping around if an overflow occurred

By being explicit, we prevent the developer from "simply not considering" how their code behaves in the presence of overflows.
In a ledger application, silently wrapping could be catastrophic, so we really want to be explicit about what behaviour we expect.

### Exceptions for `clippy`

These lints are disabled:

* `clippy::match_bool` - sometimes a `match` statement with `true =>` and `false =>` arms is sometimes more concise and equally readable
* `clippy::module_name_repetition` - warns when creating an item with a name than ends with the name of the module it's in
* `clippy::derive_partial_eq_without_eq` - warns when deriving `PartialEq` and not `Eq`.
  This is a semver hazard. Deriving `Eq` is a stronger semver guarantee than just `PartialEq`, and shouldn't be the default.
* `clippy::missing_panics_doc` - this lint warns when a function might panic, but the docs don't have a `panics` section.
  This lint is buggy, and doesn't correctly identify all panics.
  Code should be written to explicitly avoid intentional panics.
  You should still add panic docs if a function is **intended to panic** under some conditions.
  If a panic may occur, but you'd consider it a bug if it did, don't document it.
  We disable this lint because it creates a false sense of security.

## Guidelines

### Prefer references over generics

It's tempting to write a function like this:

```rust
fn use_str(s: impl AsRef<str>) {
  let s = s.as_ref();
  println!("{s}");
}
```

Unfortunately, this has a few downsides:

* it increases compile times
* if used in a trait, it makes that trait not object-safe
* if the body of the function is large, it bloats binary size, which can hurt performance by increasing pressure on the instruction cache
* it makes type inference harder

Now that's not to say you should never use generics.
Of course, there are plenty of good reasons to use generics.
But if the only reason to make your function generic is "slightly easier to use at the call-site",
consider just using a plain reference/slice instead:

```rust
fn use_str(s: &str) {
  println!("{s}");
}
```

This does mean you may have to use `.as_ref()` at the call-site, but generally this is preferred compared to the downsides of using generics.

Similar logic applies to `AsRef<Path>`, `Into<String>`, and a few other common types.
The general principle is that a little bit of extra text at the call-site is usually worth the benefits from not having generic functions.

### Abbreviations and naming things

We should be careful with abbreviations.
Similar to above, they do indeed shorten the code you write, but at the cost of some readability.
It's important to balance the readability cost against the benefits of shorter code.

Some guidelines for when to use abbreviations:

* if it's something you're going to type a lot, an abbreviation is probably the right choice.
  (i.e. `s` is an OK name for a string in very string-heavy code)
* if it's a well-known abbreviation, it's probably good (e.g. `ctx` for "context", `db` for "database")
* if it's ambiguous (i.e. it could be short for multiple things) either use the full word, or a longer abbreviation that isn't ambiguous.
* Remember that abbreviations are context-sensitive.
  (if I see `db` in a database library, it's probably "database". If I see it in an audio processing library it is probably "decibels").

#### General advice around names

* avoid `foo.get_bar()`, instead just call it `foo.bar()`
* use `into_foo()` for conversions that *consume* the original data
* use `as_foo()` for conversions that convert borrowed data to borrowed data
* use `to_foo()` for conversions that are expensive
* use `into_inner()` for extracting a wrapped value

## Pay attention to the public API of your crate

Items (functions, modules, structs, etc) should be private by default.
This is what Rust does anyways, but make sure you pay attention when marking something `pub`.

Try to keep the public API of your crate as small as possible.
It should contain *only* the items needed to provide the functionality it's responsible for.

A good "escape hatch" is to mark things as `pub(crate)`.
This makes the item `pub` but *only within your crate*.
This can be handy for "helper functions" that you want to use everywhere within your crate, but don't want to be available outside.

## Type safety

Rust has a powerful type system, so use it!

Where possible, encode important information in the type system.
For example, using `NonZeroU64` might make sense if it would be ridiculous for a number to be zero.
Of course, you can go too far with this.
Rust's type system is Turing-complete, but we don't want to write our whole program in the type system.

### Use newtypes (a.k.a. microtypes)

If you are handling email addresses, don't use `String`.
Instead, create a newtype wrapper with:

```rs
struct Email(String);
```

This prevents you from using a `Password` where you meant to use an `Email`, which catches more bugs at compile time.

Consider using the `microtype` library to generate boilerplate:

```rust
#[string]
String {
  Email,
  Username,
  Address,
  // etc...
}
```

This generates `struct Email(String)`, `struct Username(String)`, etc. for you.
See the docs for more info.

If your type is responsible for handling secret data, mark it `#[secret]` to:

* zeroize the memory on drop
* redact the `Debug` impl
* prevent serialization
* prevent use without `.expose_secret()`

### Don't over-abstract

In general, prefer plain functions over a struct + trait implementation.
For example, instead of this:

```rust
// BAD
trait GetBar {
  fn bar(&self) -> &Bar;
}

impl GetBar for Foo {
  fn bar(&self) -> &Bar {
    &self.bar
  }
}
```

write this:

```rust
// GOOD
impl Foo {
  fn bar(&self) -> &Bar {
    &self.bar
  }
}
```

I.e., don't use a trait if you don't need it.

A common reason why people do this is to mock out a particular function call for testing.
This can be useful in a few select places, such as interacting with the real world.
Eg, networking, clocks, randomness, etc. However, it has some significant downsides.

* it means you're not actually testing this code.
  This might be fine for some types of code (e.g. database code).
  It might be unreasonable to rely on a database for unit tests.
  However, if your whole test suite is organized around this, your business logic won't get tested.
* it forces you to use a trait, which have restrictions that plain functions don't have:
  * it forces you into either generics or dynamic dispatch (often with a heap allocation if you don't want to play the lifetime game)
  * you may now have to think about object safety, which can be very tricky for some APIs
  * async functions are not properly supported
  * it's not usable in a const context

Some alternative patterns are:

* try to rewrite your test to avoid needing a mock
* if you know all the variants at compile time, consider using an enum
* swap out the implementation with conditional compilation

## Unsafe code

If you need unsafe code, put it in its own crate with a safe API.
And really think hard about whether you **need** unsafe code.
There are times when you absolutely do need it, but this project cares more about **correctness** than performance.

If you find yourself wanting to use `unsafe`, try the following:

* if you want to create bindings to a C/C++ library:
  * First, see if there is a pure-Rust implementation.
  * Otherwise, search on crates.io for a `_sys` crate.
* if you want to create a cool data structure that requires unsafe:
  * does it really need unsafe?
  * is it a doubly linked list? If so, have you got benchmarks that show that a `VecDeque` is insufficient? Something something cache-friendly...
  * is there a suitable implementation on crates.io?
  * is this data structure really noticeably better than what we have in `std`?
* if you want to do a performance optimization (e.g. using `unreachable_unchecked()` to remove a bounds check):
  * Encode it in the type system, and put it in a separate crate with a safe API.
  * If you can't do that, it's probably an indication that the mental model is also too complicated for a human to keep track of.
* if you want to write a test that makes sure the code does the right thing "even in the presence of UB", just don't

**All** unsafe code **must** be tested with Miri.

## Docs

As mentioned above, we should enable `#![deny(missing_docs)]` on all new code.
But that doesn't mean we shouldn't document private items.
Ideally, we'd document as much as possible.

Of course, for tiny helper functions, or functions whose behaviour is obvious from looking don't need documentation.
For example, this sort of comment doesn't add much:

```rust
/// Sets self.bar to equal bar
fn set_bar(&mut self, bar: Bar) {
  self.bar = bar;
}
```

If this is a private member, don't bother with this comment.
If it's public, something like this is fine just to get `clippy` to shut up.
But if it's at all unclear what's going on, try to use a more descriptive comment.

If adding a dependency, add a comment explaining what the dependency does.

### Doctests

Try to use doctests.
Especially for less obvious code, a small example can be really helpful.
Humans learn by copying examples, so providing some can drastically reduce the amount of time a new contributor needs to become productive.

If you need some setup for your tests that you don't want to render in docs, prefix the line with `#`.
When combined with the `include` macro, this can lead to pretty concise but also powerful test setup.

If you need some inspiration, check out the docstests for `diesel`.

## Write code as if it's going to be in a web server

Write code as if it's going to end up being run in a web server.
This means a few things:

* **all** inputs are potentially malicious
* code should be usable as a library **without** going through a text interface (i.e. your library should expose a Rust API)

## Error handling

Error handling in Rust is complex, which represents the real-world complexity of error handling.

Broadly speaking, there are two types of error:

**Expected errors** are errors that are expected to occur during **normal operation** of the application.
For example, in bug free code, it would still be expected to see network timeout errors, since that networking is inherently fallible.
The exact error handling strategy may vary, but often involves returning a `Result`.

**Unexpected errors** are errors that are not expected to occur.
If they do occur, it represents a bug.
These errors are handled by panicking.
As much as possible, we try to make these cases impossible by construction by using the correct types for data.
For example, imagine you have a struct that represents "a list with at least one element".
You could write:

```rust
struct NonEmptyList<T> {
  inner: Vec<T>,
}

impl<T> NonEmptyList<T> {
  /// Doesn't need to be an Option<&T> because the list is guaranteed to have at least 1 element
  fn first(&self) -> &T {
    inner.get(0).expect("guaranteed to have at least 1 element")
  }
}
```

This would be *fine*, since it represents a bug if this panic is ever hit.
But it would be better to write it like this:

```rust
struct NonEmptyList<T> {
  head: T,
  tail: Vec<T>,
}

impl<T> NonEmptyList<T> {
  fn first(&self) -> &T {
    &self.head
  }
}
```

This provides the compiler with more information about the invariants of our type.
This allows us to eliminate the error at compile time.

### Handling expected errors

Well-behaved code doesn't panic.
So if our response to encountering an *expected error* is to panic, our software is not well-behaved.

Instead, we should use `Result<T, E>` to represent data that might be an error.
But how do we pick `E`?

There are two main choices for `E`:

#### Use `thiserror` for recoverable errors

In contexts where we may want to recover from errors, we should use a dedicated error type.
We generate these with `thiserror`:

```rust
#[derive(Debug, Error)]
enum FooError {
  #[error("failed to bar")]
  Bar,

  #[error("failed to baz")]
  Baz,
}
```

This allows the user to write:

```rust
match try_foo() {
  Ok(foo) => println!("got a foo: {foo}"),
  Err(FooError::Bar) => eprintln!("failed to bar"),
  Err(FooError::Baz) => eprintln!("failed to baz"),
}
```

#### Use `color_eyre` for unrecoverable errors

In contexts where we don't want to recover from errors, use `Report` from the `color_eyre` crate.
This is a trait object based error type which allows you to "fire and forget" an error.
While technically *possible*, it's less ergonomic to recover from a `Result<T, Report>`.
Therefore, only use this in contexts where the correct behaviour is "exit the program".
This is commonly the case in CLI apps.

**However**, even in CLI apps, it's good practice to split the logic into a `lib.rs` file (or modules) and have a separate binary.
