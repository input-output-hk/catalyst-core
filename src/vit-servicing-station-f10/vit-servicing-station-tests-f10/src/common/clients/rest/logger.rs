#[derive(Debug, Clone)]
pub struct RestClientLogger {
    enabled: bool,
}

impl Default for RestClientLogger {
    fn default() -> Self {
        Self { enabled: true }
    }
}

impl RestClientLogger {
    pub fn log_request(&self, request: &str) {
        if !self.is_enabled() {
            return;
        }
        println!("Request: {:#?}", request);
    }

    pub fn log_response(&self, response: &reqwest::blocking::Response) {
        if !self.is_enabled() {
            return;
        }
        println!("Response: {:#?}", response);
    }

    pub fn log_text(&self, content: &str) {
        if !self.is_enabled() {
            return;
        }
        println!("Text: {:#?}", content);
    }

    pub fn log_post_body(&self, content: &str) {
        if !self.is_enabled() {
            return;
        }
        println!("Post Body: {}", content);
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled
    }
}
