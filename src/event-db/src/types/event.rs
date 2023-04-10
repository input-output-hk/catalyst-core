use crate::types::utils::{serialize_datetime_as_rfc3339, serialize_option_datetime_as_rfc3339};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventId(pub i32);

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct EventSummary {
    pub id: EventId,
    pub name: String,
    #[serde(serialize_with = "serialize_datetime_as_rfc3339")]
    pub starts: DateTime<Utc>,
    #[serde(serialize_with = "serialize_datetime_as_rfc3339")]
    pub ends: DateTime<Utc>,
    #[serde(rename = "final")]
    pub is_final: bool,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_option_datetime_as_rfc3339"
    )]
    pub reg_checked: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct VotingPowerSettings {
    alg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    min_ada: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_pct: Option<f64>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct EventRegistration {
    #[serde(skip_serializing_if = "Option::is_none")]
    purpose: Option<i64>,
    #[serde(serialize_with = "serialize_datetime_as_rfc3339")]
    deadline: DateTime<Utc>,
    #[serde(serialize_with = "serialize_datetime_as_rfc3339")]
    taken: DateTime<Utc>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct EventGoal {
    idx: i64,
    name: String,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct EventSchedule {
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_option_datetime_as_rfc3339"
    )]
    insight_sharing: Option<DateTime<Utc>>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_option_datetime_as_rfc3339"
    )]
    proposal_submission: Option<DateTime<Utc>>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_option_datetime_as_rfc3339"
    )]
    refine_proposals: Option<DateTime<Utc>>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_option_datetime_as_rfc3339"
    )]
    finalize_proposals: Option<DateTime<Utc>>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_option_datetime_as_rfc3339"
    )]
    proposal_assessment: Option<DateTime<Utc>>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_option_datetime_as_rfc3339"
    )]
    assessment_qa_start: Option<DateTime<Utc>>,
    #[serde(serialize_with = "serialize_datetime_as_rfc3339")]
    voting: DateTime<Utc>,
    #[serde(serialize_with = "serialize_datetime_as_rfc3339")]
    tallying: DateTime<Utc>,
    #[serde(serialize_with = "serialize_datetime_as_rfc3339")]
    tallying_end: DateTime<Utc>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct VoterGroup {
    id: String,
    voting_token: String,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct EventDetails {
    pub voting_power: VotingPowerSettings,
    pub registration: EventRegistration,
    pub goals: Vec<EventGoal>,
    pub schedule: EventSchedule,
    pub groups: Vec<VoterGroup>,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct Event {
    #[serde(flatten)]
    pub event_summary: EventSummary,
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub event_details: Option<EventDetails>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use serde_json::json;

    #[test]
    fn event_id_json_test() {
        let event_ids = vec![EventId(10), EventId(11), EventId(12)];
        let json = serde_json::to_value(&event_ids).unwrap();
        assert_eq!(json, json!([10, 11, 12]));

        let decoded: Vec<EventId> = serde_json::from_value(json).unwrap();
        assert_eq!(decoded, event_ids);
    }

    #[test]
    fn event_summary_json_test() {
        let event_summary = EventSummary {
            id: EventId(1),
            name: "Fund 10".to_string(),
            starts: DateTime::from_utc(NaiveDateTime::from_timestamp_opt(0, 0).unwrap(), Utc),
            ends: DateTime::from_utc(NaiveDateTime::from_timestamp_opt(0, 0).unwrap(), Utc),
            is_final: true,
            reg_checked: Some(DateTime::from_utc(
                NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                Utc,
            )),
        };

        let json = serde_json::to_value(&event_summary).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "id": 1,
                    "name": "Fund 10",
                    "starts": "1970-01-01T00:00:00+00:00",
                    "ends": "1970-01-01T00:00:00+00:00",
                    "final": true,
                    "reg_checked": "1970-01-01T00:00:00+00:00",
                }
            )
        );

        let event_summary = EventSummary {
            id: EventId(1),
            name: "Fund 10".to_string(),
            starts: DateTime::from_utc(NaiveDateTime::from_timestamp_opt(0, 0).unwrap(), Utc),
            ends: DateTime::from_utc(NaiveDateTime::from_timestamp_opt(0, 0).unwrap(), Utc),
            is_final: true,
            reg_checked: None,
        };

        let json = serde_json::to_value(&event_summary).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "id": 1,
                    "name": "Fund 10",
                    "starts": "1970-01-01T00:00:00+00:00",
                    "ends": "1970-01-01T00:00:00+00:00",
                    "final": true,
                }
            )
        );
    }

    #[test]
    fn voting_power_settings_json_test() {
        let voting_power_settings = VotingPowerSettings {
            alg: "threshold_staked_ADA".to_string(),
            min_ada: Some(500),
            max_pct: Some(1.23),
        };

        let json = serde_json::to_value(&voting_power_settings).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "alg": "threshold_staked_ADA",
                    "min_ada": 500,
                    "max_pct": 1.23,
                }
            )
        );

        let voting_power_settings = VotingPowerSettings {
            alg: "threshold_staked_ADA".to_string(),
            min_ada: None,
            max_pct: None,
        };

        let json = serde_json::to_value(&voting_power_settings).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "alg": "threshold_staked_ADA",
                }
            )
        );
    }

    #[test]
    fn event_registration_json_test() {
        let event_registration = EventRegistration {
            purpose: Some(1),
            deadline: DateTime::from_utc(NaiveDateTime::from_timestamp_opt(0, 0).unwrap(), Utc),
            taken: DateTime::from_utc(NaiveDateTime::from_timestamp_opt(0, 0).unwrap(), Utc),
        };

        let json = serde_json::to_value(&event_registration).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "purpose": 1,
                    "deadline": "1970-01-01T00:00:00+00:00",
                    "taken": "1970-01-01T00:00:00+00:00",
                }
            )
        );

        let event_registration = EventRegistration {
            purpose: None,
            deadline: DateTime::from_utc(NaiveDateTime::from_timestamp_opt(0, 0).unwrap(), Utc),
            taken: DateTime::from_utc(NaiveDateTime::from_timestamp_opt(0, 0).unwrap(), Utc),
        };

        let json = serde_json::to_value(&event_registration).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "deadline": "1970-01-01T00:00:00+00:00",
                    "taken": "1970-01-01T00:00:00+00:00",
                }
            )
        );
    }

    #[test]
    fn event_goal_json_test() {
        let event_goal = EventGoal {
            idx: 1,
            name: "goal 1".to_string(),
        };

        let json = serde_json::to_value(&event_goal).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "idx": 1,
                    "name": "goal 1",
                }
            )
        );
    }

    #[test]
    fn event_schedule_json_test() {
        let event_schedule = EventSchedule {
            insight_sharing: Some(DateTime::from_utc(
                NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                Utc,
            )),
            proposal_submission: Some(DateTime::from_utc(
                NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                Utc,
            )),
            refine_proposals: Some(DateTime::from_utc(
                NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                Utc,
            )),
            finalize_proposals: Some(DateTime::from_utc(
                NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                Utc,
            )),
            proposal_assessment: Some(DateTime::from_utc(
                NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                Utc,
            )),
            assessment_qa_start: Some(DateTime::from_utc(
                NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                Utc,
            )),
            voting: DateTime::from_utc(NaiveDateTime::from_timestamp_opt(0, 0).unwrap(), Utc),
            tallying: DateTime::from_utc(NaiveDateTime::from_timestamp_opt(0, 0).unwrap(), Utc),
            tallying_end: DateTime::from_utc(NaiveDateTime::from_timestamp_opt(0, 0).unwrap(), Utc),
        };

        let json = serde_json::to_value(&event_schedule).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "insight_sharing": "1970-01-01T00:00:00+00:00",
                    "proposal_submission": "1970-01-01T00:00:00+00:00",
                    "refine_proposals": "1970-01-01T00:00:00+00:00",
                    "finalize_proposals": "1970-01-01T00:00:00+00:00",
                    "proposal_assessment": "1970-01-01T00:00:00+00:00",
                    "assessment_qa_start": "1970-01-01T00:00:00+00:00",
                    "voting": "1970-01-01T00:00:00+00:00",
                    "tallying": "1970-01-01T00:00:00+00:00",
                    "tallying_end": "1970-01-01T00:00:00+00:00",
                }
            )
        );

        let event_schedule = EventSchedule {
            insight_sharing: None,
            proposal_submission: None,
            refine_proposals: None,
            finalize_proposals: None,
            proposal_assessment: None,
            assessment_qa_start: None,
            voting: DateTime::from_utc(NaiveDateTime::from_timestamp_opt(0, 0).unwrap(), Utc),
            tallying: DateTime::from_utc(NaiveDateTime::from_timestamp_opt(0, 0).unwrap(), Utc),
            tallying_end: DateTime::from_utc(NaiveDateTime::from_timestamp_opt(0, 0).unwrap(), Utc),
        };

        let json = serde_json::to_value(&event_schedule).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "voting": "1970-01-01T00:00:00+00:00",
                    "tallying": "1970-01-01T00:00:00+00:00",
                    "tallying_end": "1970-01-01T00:00:00+00:00",
                }
            )
        );
    }

    #[test]
    fn voter_group_json_test() {
        let voter_group = VoterGroup {
            id: "rep".to_string(),
            voting_token: "voting token 1".to_string(),
        };

        let json = serde_json::to_value(&voter_group).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "id": "rep",
                    "voting_token": "voting token 1",
                }
            )
        );
    }

    #[test]
    fn event_details_json_test() {
        let event_details = EventDetails {
            voting_power: VotingPowerSettings {
                alg: "threshold_staked_ADA".to_string(),
                min_ada: Some(500),
                max_pct: Some(1.23),
            },
            registration: EventRegistration {
                purpose: Some(1),
                deadline: DateTime::from_utc(NaiveDateTime::from_timestamp_opt(0, 0).unwrap(), Utc),
                taken: DateTime::from_utc(NaiveDateTime::from_timestamp_opt(0, 0).unwrap(), Utc),
            },
            goals: vec![EventGoal {
                idx: 1,
                name: "goal 1".to_string(),
            }],
            schedule: EventSchedule {
                insight_sharing: Some(DateTime::from_utc(
                    NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                    Utc,
                )),
                proposal_submission: Some(DateTime::from_utc(
                    NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                    Utc,
                )),
                refine_proposals: Some(DateTime::from_utc(
                    NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                    Utc,
                )),
                finalize_proposals: Some(DateTime::from_utc(
                    NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                    Utc,
                )),
                proposal_assessment: Some(DateTime::from_utc(
                    NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                    Utc,
                )),
                assessment_qa_start: Some(DateTime::from_utc(
                    NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                    Utc,
                )),
                voting: DateTime::from_utc(NaiveDateTime::from_timestamp_opt(0, 0).unwrap(), Utc),
                tallying: DateTime::from_utc(NaiveDateTime::from_timestamp_opt(0, 0).unwrap(), Utc),
                tallying_end: DateTime::from_utc(
                    NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                    Utc,
                ),
            },
            groups: vec![VoterGroup {
                id: "rep".to_string(),
                voting_token: "voting token 1".to_string(),
            }],
        };

        let json = serde_json::to_value(&event_details).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "voting_power": {
                                        "alg": "threshold_staked_ADA",
                                        "min_ada": 500,
                                        "max_pct": 1.23,
                                    },
                    "registration": {
                                        "purpose": 1,
                                        "deadline": "1970-01-01T00:00:00+00:00",
                                        "taken": "1970-01-01T00:00:00+00:00",
                                    },
                    "goals": [
                                {
                                    "idx": 1,
                                    "name": "goal 1",
                                }
                            ],
                    "schedule": {
                                    "insight_sharing": "1970-01-01T00:00:00+00:00",
                                    "proposal_submission": "1970-01-01T00:00:00+00:00",
                                    "refine_proposals": "1970-01-01T00:00:00+00:00",
                                    "finalize_proposals": "1970-01-01T00:00:00+00:00",
                                    "proposal_assessment": "1970-01-01T00:00:00+00:00",
                                    "assessment_qa_start": "1970-01-01T00:00:00+00:00",
                                    "voting": "1970-01-01T00:00:00+00:00",
                                    "tallying": "1970-01-01T00:00:00+00:00",
                                    "tallying_end": "1970-01-01T00:00:00+00:00",
                            },
                    "groups": [
                                {
                                    "id": "rep",
                                    "voting_token": "voting token 1",
                                }
                            ],
                }
            )
        );
    }

    #[test]
    fn event_json_test() {
        let event_summary = Event {
            event_summary: EventSummary {
                id: EventId(1),
                name: "Fund 10".to_string(),
                starts: DateTime::from_utc(NaiveDateTime::from_timestamp_opt(0, 0).unwrap(), Utc),
                ends: DateTime::from_utc(NaiveDateTime::from_timestamp_opt(0, 0).unwrap(), Utc),
                is_final: true,
                reg_checked: Some(DateTime::from_utc(
                    NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                    Utc,
                )),
            },
            event_details: Some(EventDetails {
                voting_power: VotingPowerSettings {
                    alg: "threshold_staked_ADA".to_string(),
                    min_ada: Some(500),
                    max_pct: Some(1.23),
                },
                registration: EventRegistration {
                    purpose: Some(1),
                    deadline: DateTime::from_utc(
                        NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                        Utc,
                    ),
                    taken: DateTime::from_utc(
                        NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                        Utc,
                    ),
                },
                goals: vec![EventGoal {
                    idx: 1,
                    name: "goal 1".to_string(),
                }],
                schedule: EventSchedule {
                    insight_sharing: Some(DateTime::from_utc(
                        NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                        Utc,
                    )),
                    proposal_submission: Some(DateTime::from_utc(
                        NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                        Utc,
                    )),
                    refine_proposals: Some(DateTime::from_utc(
                        NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                        Utc,
                    )),
                    finalize_proposals: Some(DateTime::from_utc(
                        NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                        Utc,
                    )),
                    proposal_assessment: Some(DateTime::from_utc(
                        NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                        Utc,
                    )),
                    assessment_qa_start: Some(DateTime::from_utc(
                        NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                        Utc,
                    )),
                    voting: DateTime::from_utc(
                        NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                        Utc,
                    ),
                    tallying: DateTime::from_utc(
                        NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                        Utc,
                    ),
                    tallying_end: DateTime::from_utc(
                        NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                        Utc,
                    ),
                },
                groups: vec![VoterGroup {
                    id: "rep".to_string(),
                    voting_token: "voting token 1".to_string(),
                }],
            }),
        };

        let json = serde_json::to_value(&event_summary).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "id": 1,
                    "name": "Fund 10",
                    "starts": "1970-01-01T00:00:00+00:00",
                    "ends": "1970-01-01T00:00:00+00:00",
                    "final": true,
                    "reg_checked": "1970-01-01T00:00:00+00:00",
                    "voting_power": {
                                        "alg": "threshold_staked_ADA",
                                        "min_ada": 500,
                                        "max_pct": 1.23,
                                    },
                    "registration": {
                                        "purpose": 1,
                                        "deadline": "1970-01-01T00:00:00+00:00",
                                        "taken": "1970-01-01T00:00:00+00:00",
                                    },
                    "goals": [
                                {
                                    "idx": 1,
                                    "name": "goal 1",
                                }
                            ],
                    "schedule": {
                                    "insight_sharing": "1970-01-01T00:00:00+00:00",
                                    "proposal_submission": "1970-01-01T00:00:00+00:00",
                                    "refine_proposals": "1970-01-01T00:00:00+00:00",
                                    "finalize_proposals": "1970-01-01T00:00:00+00:00",
                                    "proposal_assessment": "1970-01-01T00:00:00+00:00",
                                    "assessment_qa_start": "1970-01-01T00:00:00+00:00",
                                    "voting": "1970-01-01T00:00:00+00:00",
                                    "tallying": "1970-01-01T00:00:00+00:00",
                                    "tallying_end": "1970-01-01T00:00:00+00:00",
                            },
                    "groups": [
                                {
                                    "id": "rep",
                                    "voting_token": "voting token 1",
                                }
                            ],
                }
            )
        );
    }
}
