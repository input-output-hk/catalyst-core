use self::{
    event::{
        objective::ObjectiveQueries, proposal::ProposalQueries, review::ReviewQueries, EventQueries,
    },
    registration::RegistrationQueries,
    search::SearchQueries,
};
use crate::EventDB;

pub mod event;
pub mod registration;
pub mod search;

pub trait EventDbQueries:
    RegistrationQueries
    + EventQueries
    + ObjectiveQueries
    + ProposalQueries
    + ReviewQueries
    + SearchQueries
{
}

impl EventDbQueries for EventDB {}
