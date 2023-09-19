use super::{objective_details::ObjectiveDetails, objective_summary::ObjectiveSummary};
use poem_openapi::{types::Example, Object};

/// Full Objective info.
#[derive(Object)]
pub(crate) struct Objective {
    #[oai(flatten)]
    summary: ObjectiveSummary,
    #[oai(flatten)]
    details: ObjectiveDetails,
}

impl Example for Objective {
    fn example() -> Self {
        Self {
            summary: ObjectiveSummary::example(),
            details: ObjectiveDetails::example(),
        }
    }
}

impl TryFrom<event_db::types::objective::Objective> for Objective {
    type Error = String;
    fn try_from(value: event_db::types::objective::Objective) -> Result<Self, Self::Error> {
        Ok(Self {
            summary: value.summary.try_into()?,
            details: value.details.try_into()?,
        })
    }
}
