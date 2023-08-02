#[cfg(test)]
#[macro_use(lazy_static)]
extern crate lazy_static;

pub mod context;
#[cfg(test)]
pub mod jcli;
#[cfg(test)]
#[allow(clippy::all)]
pub mod jormungandr;
#[cfg(all(test, feature = "network"))]
#[allow(clippy::all)]
pub mod networking;
#[cfg(all(test, feature = "non-functional"))]
#[allow(clippy::all)]
pub mod non_functional;
#[allow(clippy::all)]
pub mod startup;
