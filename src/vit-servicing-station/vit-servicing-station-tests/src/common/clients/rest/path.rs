use url::Url;

#[derive(Debug, Clone)]
pub struct RestPathBuilder {
    address: Url,
    root: String,
}

impl RestPathBuilder {
    pub fn new(address: Url) -> Self {
        RestPathBuilder {
            root: "api/v0/".to_string(),
            address,
        }
    }

    pub fn admin(self) -> Self {
        Self {
            address: self.address,
            root: self.root + "admin/",
        }
    }

    pub fn proposals(&self) -> String {
        self.path("proposals")
    }

    pub fn proposals_with_group(&self, group: &str) -> String {
        self.path(&format!("proposals/{}", group))
    }

    pub fn funds(&self) -> String {
        self.path("fund")
    }

    pub fn challenges(&self) -> String {
        self.path("challenges")
    }

    pub fn snapshot_info(&self, tag: &str) -> String {
        self.path(&format!("snapshot/snapshot_info/{}", tag))
    }

    pub fn raw_snapshot(&self, tag: &str) -> String {
        self.path(&format!("snapshot/raw_snapshot/{}", tag))
    }

    pub fn snapshot_tags(&self) -> String {
        self.path("snapshot")
    }

    pub fn snapshot_voter_info(&self, tag: &str, key: &str) -> String {
        self.path(&format!("snapshot/voter/{}/{}", tag, key))
    }

    pub fn snapshot_delegator_info(&self, tag: &str, key: &str) -> String {
        self.path(&format!("snapshot/delegator/{}/{}", tag, key))
    }

    pub fn proposal(&self, id: &str, group: &str) -> String {
        self.path(&format!("proposal/{}/{}", id, group))
    }

    pub fn fund(&self, id: &str) -> String {
        self.path(&format!("fund/{}", id))
    }

    pub fn advisor_reviews(&self, id: &str) -> String {
        self.path(&format!("reviews/{}", id))
    }

    pub fn genesis(&self) -> String {
        self.path("block0")
    }

    pub fn health(&self) -> String {
        self.path("health")
    }

    pub fn service_version(&self) -> String {
        format!("{}{}{}", self.address, "api/", "vit-version")
    }

    pub fn search(&self) -> String {
        self.path("search")
    }

    pub fn search_count(&self) -> String {
        self.path("search_count")
    }

    pub fn path(&self, path: &str) -> String {
        format!("{}{}{}", self.address, self.root, path)
    }
}
