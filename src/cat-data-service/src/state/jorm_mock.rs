use crate::types::jorm_mock::{Fragment, FragmentsProcessingSummary};

#[derive(Default)]
pub struct JormState {}

impl JormState {
    pub fn accept_fragments(
        &self,
        _fail_fast: bool,
        _fragments: Vec<Fragment>,
    ) -> FragmentsProcessingSummary {

        
        FragmentsProcessingSummary {
            accepted: vec![],
            rejected: vec![],
        }
    }
}
