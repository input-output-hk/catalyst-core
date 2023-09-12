//! This module contains common and re-usable responses with a 4xx response code.
//!

use poem::IntoResponse;
use poem_extensions::OneResponse;
use poem_openapi::payload::{Payload, PlainText};

#[derive(OneResponse)]
#[oai(status = 400)]
pub struct BadRequest<T: IntoResponse + Payload>(T);

#[derive(OneResponse)]
#[oai(status = 400)]
/// This error means that the request was malformed.
/// It has failed to pass validation, as specified by the OpenAPI schema.
pub struct ApiValidationError(PlainText<String>);

#[derive(OneResponse)]
#[oai(status = 401)]
pub struct Unauthorized;

#[derive(OneResponse)]
#[oai(status = 403)]
pub struct Forbidden;

#[derive(OneResponse)]
#[oai(status = 404)]
/// ## Content not found
pub struct NotFound;

#[derive(OneResponse)]
#[oai(status = 405)]
pub struct MethodNotAllowed;

#[derive(OneResponse)]
#[oai(status = 406)]
pub struct NotAcceptable;

#[derive(OneResponse)]
#[oai(status = 422)]
/// Common automatically produced validation error for every endpoint.
/// Is generated automatically when any of the OpenAPI validation rules fail.
/// Can also be generated manually.
pub struct ValidationError;
