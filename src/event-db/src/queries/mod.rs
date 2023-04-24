use self::{
    event::{objective::ObjectiveQueries, EventQueries},
    registration::RegistrationQueries,
};
use crate::EventDB;

pub mod event;
pub mod registration;

pub trait EventDbQueries: RegistrationQueries + EventQueries + ObjectiveQueries {}

impl EventDbQueries for EventDB {}
