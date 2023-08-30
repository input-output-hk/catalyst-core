//! This module contains generic re-usable responses with a 2xx response code.
//!

use poem_extensions::OneResponse;

#[derive(OneResponse)]
#[oai(status = 200)]
pub(crate) struct EmptyOK;

#[derive(OneResponse)]
#[oai(status = 204)]
pub(crate) struct NoContent;
