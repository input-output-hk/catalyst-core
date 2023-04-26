use self::{
    event::{objective::ObjectiveQueries, proposal::ProposalQueries, EventQueries},
    registration::RegistrationQueries,
};
use crate::EventDB;

pub mod event;
pub mod registration;

pub trait EventDbQueries:
    RegistrationQueries + EventQueries + ObjectiveQueries + ProposalQueries
{
}

impl EventDbQueries for EventDB {}
