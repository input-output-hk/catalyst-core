use crate::EventDB;

use self::{event::EventQueries, registration::RegistrationQueries};

pub mod event;
pub mod registration;

pub trait EventDbQueries: RegistrationQueries + EventQueries {}

impl EventDbQueries for EventDB {}
