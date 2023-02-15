// Needed for election-db schema.rs diesel macros to be expanded
#![recursion_limit = "256"]

pub mod common;

#[cfg(test)]
pub mod tests;

#[cfg(test)]
extern crate lazy_static;

#[macro_use]
extern crate diesel;
