use chrono::{DateTime, SecondsFormat, Utc};
use jormungandr_automation::jormungandr::{JormungandrRest, RestError};
use jormungandr_lib::interfaces::FragmentLog;
use serde_json;

pub struct Harvester {
    rest: JormungandrRest,
    endpoint: String,
    timeout: std::time::Duration,
}

impl Harvester {
    pub fn new<S: Into<String>>(endpoint: S, timeout: std::time::Duration) -> Self {
        let endpoint = endpoint.into();

        Self {
            rest: JormungandrRest::new(endpoint.clone()),
            timeout,
            endpoint,
        }
    }

    pub fn harvest(&self) -> Result<Snapshot, RestError> {
        let mut votes_count: usize = 0;

        for vote_status in self.rest.vote_plan_statuses()? {
            votes_count += vote_status
                .proposals
                .iter()
                .map(|x| x.votes_cast)
                .sum::<usize>();
        }

        let fragment_logs = self.fragment_logs();

        Ok(Snapshot {
            timestamp: Utc::now(),
            pending: fragment_logs.iter().filter(|x| x.is_pending()).count(),
            total_tx: fragment_logs.len(),
            votes_count,
        })
    }

    pub fn fragment_logs(&self) -> Vec<FragmentLog> {
        let client = reqwest::blocking::Client::builder()
            .timeout(self.timeout)
            .build()
            .unwrap();

        let res = client
            .get(&format!("{}/v0/fragment/logs", self.endpoint))
            .send()
            .unwrap();
        serde_json::from_str(&res.text().unwrap()).unwrap()
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
