use self::{
    event::{
        objective::ObjectiveQueries, proposal::ProposalQueries, review::ReviewQueries, EventQueries,
    },
    registration::RegistrationQueries,
};
use crate::EventDB;

pub mod event;
pub mod registration;
pub mod search;

pub trait EventDbQueries:
    RegistrationQueries + EventQueries + ObjectiveQueries + ProposalQueries + ReviewQueries
{
}

impl EventDbQueries for EventDB {}
