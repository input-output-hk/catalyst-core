use chrono::{DateTime, SecondsFormat, Utc};
use jormungandr_testing_utils::testing::node::JormungandrRest;

pub struct Harvester {
    rest: JormungandrRest,
}

impl Harvester {
    pub fn new<S: Into<String>>(endpoint: S) -> Self {
        Self {
            rest: JormungandrRest::new(endpoint.into()),
        }
    }

    pub fn harvest(&self) -> Result<Snapshot, jormungandr_testing_utils::testing::node::RestError> {
        let mut votes_count: usize = 0;

        for vote_status in self.rest.vote_plan_statuses()? {
            votes_count += vote_status
                .proposals
                .iter()
                .map(|x| x.votes_cast)
                .sum::<usize>();
        }

        let fragment_logs = self.rest.fragment_logs()?;

        Ok(Snapshot {
            timestamp: Utc::now(),
            pending: fragment_logs.iter().filter(|(_, x)| x.is_pending()).count(),
            total_tx: fragment_logs.len(),
            votes_count,
        })
    }
}

pub struct Snapshot {
    pub timestamp: DateTime<Utc>,
    pub votes_count: usize,
    pub pending: usize,
    pub total_tx: usize,
}

impl Snapshot {
    pub fn header(&self) -> String {
        "date,\tvotes-count,\tpending,\ttotal-tx".to_string()
    }

    pub fn entry(&self) -> String {
        format!(
            "{},\t{},\t{},\t{}",
            self.timestamp.to_rfc3339_opts(SecondsFormat::Secs, true),
            self.votes_count,
            self.pending,
            self.total_tx
        )
    }

    pub fn to_console_output(&self) -> String {
        format!(
            "date: {},\tvotes-count: {},\tpending: {},\ttotal-tx: {}",
            self.timestamp.to_rfc3339_opts(SecondsFormat::Secs, true),
            self.votes_count,
            self.pending,
            self.total_tx
        )
    }
}
