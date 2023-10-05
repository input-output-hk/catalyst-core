#[derive(Debug, Clone, Default)]
pub struct ProposalConfig {
    pub(crate) funds: Option<i64>,
    pub(crate) challenge_id: Option<usize>,
}

impl ProposalConfig {
    pub fn funds(mut self, funds: i64) -> Self {
        self.funds = Some(funds);
        self
    }
}
