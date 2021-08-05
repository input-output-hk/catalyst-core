use once_cell::unsync::OnceCell;
use serde::Deserialize;
use std::collections::HashMap;
use vit_servicing_station_lib::db::models::community_advisors_reviews::ReviewTag;

#[derive(Deserialize, Debug, Clone)]
pub struct TagsMap {
    pub setting_alignment: String,
    pub setting_verifiability: String,
    pub setting_feasibility: String,
    pub standard_impact: String,
    pub standard_feasibility: String,
    pub standard_auditability: String,
    #[serde(default, skip_deserializing)]
    tags_map: OnceCell<HashMap<String, ReviewTag>>,
}

impl Default for TagsMap {
    fn default() -> Self {
        let setting_alignment =
            "This challenge is critical to achieve Cardano's mission".to_string();
        let setting_verifiability = "Success criteria and suggested metrics are set correctly to measure progress in addressing the challenge".to_string();
        let setting_feasibility =
            "The Catalyst community has the capacity to address the challenge".to_string();
        let standard_impact = "This proposal effectively addresses the challenge".to_string();
        let standard_feasibility = "Given experience and plan presented it is highly likely this proposal will be implemented successfully".to_string();
        let standard_auditability = "The information provided is sufficient to audit the progress and the success of the proposal".to_string();
        Self::new(
            setting_alignment,
            setting_verifiability,
            setting_feasibility,
            standard_impact,
            standard_feasibility,
            standard_auditability,
        )
    }
}

impl TagsMap {
    pub fn new(
        setting_alignment: String,
        setting_verifiability: String,
        setting_feasibility: String,
        standard_impact: String,
        standard_feasibility: String,
        standard_auditability: String,
    ) -> Self {
        Self {
            setting_alignment,
            setting_verifiability,
            setting_feasibility,
            standard_impact,
            standard_feasibility,
            standard_auditability,
            tags_map: OnceCell::new(),
        }
    }

    fn initialize_inner_map(&self) -> HashMap<String, ReviewTag> {
        let mut tags_map = HashMap::new();
        tags_map.insert(self.setting_alignment.clone(), ReviewTag::Alignment);
        tags_map.insert(self.setting_verifiability.clone(), ReviewTag::Verifiability);
        tags_map.insert(self.setting_feasibility.clone(), ReviewTag::Feasibility);
        tags_map.insert(self.standard_impact.clone(), ReviewTag::Impact);
        tags_map.insert(self.standard_feasibility.clone(), ReviewTag::Feasibility);
        tags_map.insert(self.standard_auditability.clone(), ReviewTag::Auditability);
        tags_map
    }

    pub fn str_to_tag(&self, key: &str) -> Option<ReviewTag> {
        self.tags_map
            .get_or_init(|| self.initialize_inner_map())
            .get(key)
            .copied()
    }
}
