use chrono::{DateTime, Utc};
use poem_openapi::{Enum, NewType, Object};
use rust_decimal::prelude::ToPrimitive;
use serde::Deserialize;

/// The Numeric ID of a Voting Event.
#[derive(NewType, Deserialize)]
pub struct EventId(pub i32);

impl From<EventId> for event_db::types::event::EventId {
    fn from(event_id: EventId) -> Self {
        event_db::types::event::EventId(event_id.0)
    }
}

impl From<event_db::types::event::EventId> for EventId {
    fn from(event_id: event_db::types::event::EventId) -> Self {
        Self(event_id.0)
    }
}

/// The Name of a Voting Event.
#[derive(NewType)]
pub struct EventName(pub String);

/// A Summary of an individual Voting Event.
#[derive(Object)]
pub struct EventSummary {
    id: EventId,
    name: EventName,

    /// Date-Time when the Voting Event commences.
    #[oai(skip_serializing_if_is_none = true)]
    starts: Option<DateTime<Utc>>,

    /// Date-Time when the Voting Event is expected to finish.
    #[oai(skip_serializing_if_is_none = true)]
    ends: Option<DateTime<Utc>>,

    /// Last time registrations and Voting power were checked.
    /// If not present, no registration or voting power records exist for this event.
    #[oai(skip_serializing_if_is_none = true)]
    reg_checked: Option<DateTime<Utc>>,

    /// True if the event is finished and no changes can be made to it.
    /// Does not Including payment of rewards or funding of projects.
    #[oai(rename = "final")]
    is_final: bool,
}

impl From<event_db::types::event::EventSummary> for EventSummary {
    fn from(value: event_db::types::event::EventSummary) -> Self {
        Self {
            id: value.id.into(),
            name: EventName(value.name),
            starts: value.starts.map(Into::into),
            ends: value.ends.map(Into::into),
            reg_checked: value.reg_checked.map(Into::into),
            is_final: value.is_final,
        }
    }
}

/// The Voting Power Algorithm.
#[derive(Enum)]
pub enum VotingPowerAlgorithm {
    /// Linear Voting Power in Staked ADA, With a minimum limit and maximum relative threshold.
    #[oai(rename = "threshold_staked_ADA")]
    ThresholdStakedADA,
}

impl From<event_db::types::event::VotingPowerAlgorithm> for VotingPowerAlgorithm {
    fn from(value: event_db::types::event::VotingPowerAlgorithm) -> Self {
        match value {
            event_db::types::event::VotingPowerAlgorithm::ThresholdStakedADA => {
                VotingPowerAlgorithm::ThresholdStakedADA
            }
        }
    }
}

/// The Settings Used to configure the voting power calculation.
#[derive(Object)]
struct VotingPowerSettings {
    alg: VotingPowerAlgorithm,

    /// Minimum staked funds required for a valid voter registration.
    /// This amount is in Whole ADA. If not present, there is no minimum.
    ///
    /// Valid for `alg`:
    /// * `threshold_staked_ADA`
    #[oai(skip_serializing_if_is_none = true)]
    min_ada: Option<i64>,

    /// Maximum Percentage of total registered voting power allowed for voting power.
    /// For example `1.23` = `1.23%` of total registered staked ADA as maximum voting power.
    /// If not present, there is no maximum percentage.
    ///
    /// Valid for `alg`:
    /// * `threshold_staked_ADA`
    #[oai(skip_serializing_if_is_none = true)]
    max_pct: Option<f64>,
}

impl TryFrom<event_db::types::event::VotingPowerSettings> for VotingPowerSettings {
    type Error = String;
    fn try_from(value: event_db::types::event::VotingPowerSettings) -> Result<Self, Self::Error> {
        Ok(Self {
            alg: value.alg.into(),
            min_ada: value.min_ada,
            max_pct: if let Some(max_pct) = value.max_pct {
                Some(
                    max_pct
                        .to_f64()
                        .ok_or_else(|| format!("cannot convert decimal to f64: {}", max_pct))?,
                )
            } else {
                None
            },
        })
    }
}

/// Details about Voting Event Registration.
#[derive(Object)]
struct EventRegistration {
    /// The Registration Purpose.
    #[oai(skip_serializing_if_is_none = true)]
    purpose: Option<i64>,

    /// The deadline for Registration/Voting Power to be fixed.
    /// Changes to Registrations or Voting power after this time are not considered.
    #[oai(skip_serializing_if_is_none = true)]
    deadline: Option<DateTime<Utc>>,

    /// The time after which Final Registration/Voting Power will be calculated.
    /// This is usually after the deadline, to allow for potential instability in the head of the blockchain to stabilize.
    #[oai(skip_serializing_if_is_none = true)]
    taken: Option<DateTime<Utc>>,
}

impl From<event_db::types::event::EventRegistration> for EventRegistration {
    fn from(value: event_db::types::event::EventRegistration) -> Self {
        Self {
            purpose: value.purpose,
            deadline: value.deadline.map(Into::into),
            taken: value.taken.map(Into::into),
        }
    }
}

/// An Individual Event Goal.
#[derive(Object)]
struct EventGoal {
    /// The Relative order of this Goal. 0 being highest.
    idx: i32,

    /// The name/short description of the goal.
    name: String,
}

impl From<event_db::types::event::EventGoal> for EventGoal {
    fn from(value: event_db::types::event::EventGoal) -> Self {
        Self {
            idx: value.idx,
            name: value.name,
        }
    }
}

/// The chronological sequence of stages of the voting event.
/// Stages run chronologically and only 1 stage can run at a time.
/// Each new stage terminates the previous stage.
/// Any omitted entries are assumed to not exist as a stage in this event.
#[derive(Object)]
struct EventSchedule {
    /// Date-Time when Insight Sharing Starts.
    #[oai(skip_serializing_if_is_none = true)]
    insight_sharing: Option<DateTime<Utc>>,

    /// Date-Time when Proposals can be submitted to the Voting Event.
    #[oai(skip_serializing_if_is_none = true)]
    proposal_submission: Option<DateTime<Utc>>,

    /// Date-Time when Proposal refinement begins.
    #[oai(skip_serializing_if_is_none = true)]
    refine_proposals: Option<DateTime<Utc>>,

    /// Date-Time when Proposal Finalization starts.
    #[oai(skip_serializing_if_is_none = true)]
    finalize_proposals: Option<DateTime<Utc>>,

    /// Date-Time when Proposal Assessment starts.
    #[oai(skip_serializing_if_is_none = true)]
    proposal_assessment: Option<DateTime<Utc>>,

    /// Date-Time when Assessment QA starts.
    #[oai(skip_serializing_if_is_none = true)]
    assessment_qa_start: Option<DateTime<Utc>>,

    /// Date-Time when Voting commences.
    #[oai(skip_serializing_if_is_none = true)]
    voting: Option<DateTime<Utc>>,

    /// Date-Time when Voting ends and tallying commences.
    #[oai(skip_serializing_if_is_none = true)]
    tallying: Option<DateTime<Utc>>,

    /// Date-Time when Tallying Ends.
    #[oai(skip_serializing_if_is_none = true)]
    tallying_end: Option<DateTime<Utc>>,
}

impl From<event_db::types::event::EventSchedule> for EventSchedule {
    fn from(value: event_db::types::event::EventSchedule) -> Self {
        Self {
            insight_sharing: value.insight_sharing.map(Into::into),
            proposal_submission: value.proposal_submission.map(Into::into),
            refine_proposals: value.refine_proposals.map(Into::into),
            finalize_proposals: value.finalize_proposals.map(Into::into),
            proposal_assessment: value.proposal_assessment.map(Into::into),
            assessment_qa_start: value.assessment_qa_start.map(Into::into),
            voting: value.voting.map(Into::into),
            tallying: value.tallying.map(Into::into),
            tallying_end: value.tallying_end.map(Into::into),
        }
    }
}

/// Detailed information for an individual voting event.
#[derive(Object)]
struct EventDetails {
    /// How Voting Power is Calculated and its parameters.
    voting_power_settings: VotingPowerSettings,

    /// Registration deadlines and when its finalized.  Plus any other parameters.
    registration: EventRegistration,

    ///  The schedule of the voting Event.
    schedule: EventSchedule,

    /// Description of the voting events goals.
    /// If this field is not present, there are no listed goals for the event.
    goals: Vec<EventGoal>,
}

impl TryFrom<event_db::types::event::EventDetails> for EventDetails {
    type Error = String;
    fn try_from(value: event_db::types::event::EventDetails) -> Result<Self, Self::Error> {
        Ok(Self {
            voting_power_settings: value.voting_power.try_into()?,
            registration: value.registration.into(),
            schedule: value.schedule.into(),
            goals: value.goals.into_iter().map(Into::into).collect(),
        })
    }
}

/// Complete Details about an individual Voting Event.
#[derive(Object)]
pub struct Event {
    #[oai(flatten)]
    summary: EventSummary,
    #[oai(flatten)]
    details: EventDetails,
}

impl TryFrom<event_db::types::event::Event> for Event {
    type Error = String;
    fn try_from(value: event_db::types::event::Event) -> Result<Self, Self::Error> {
        Ok(Self {
            summary: value.summary.into(),
            details: value.details.try_into()?,
        })
    }
}
