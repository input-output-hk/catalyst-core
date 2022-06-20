use crate::http::HttpClient;
use crate::ideascale::models::de::{Fund, Funnel, Proposal, Stage};

use color_eyre::Report;
use reqwest::StatusCode;
use std::collections::HashMap;

pub type Scores = HashMap<u32, f32>;
pub type Sponsors = HashMap<String, String>;

pub fn get_funds_data(client: &impl HttpClient) -> Result<Vec<Fund>, Report> {
    client.get("campaigns/groups")?.json()
}

pub fn get_stages(client: &impl HttpClient) -> Result<Vec<Stage>, Report> {
    client.get("stages")?.json()
}

/// we test token by running lightweight query and observe response code
pub fn is_token_valid(client: &impl HttpClient) -> Result<bool, Report> {
    Ok(client.get::<()>("profile/avatars")?.status() == StatusCode::OK)
}

pub fn get_proposals_data(
    client: &impl HttpClient,
    challenge_id: u32,
) -> Result<Vec<Proposal>, Report> {
    let path = &format!("campaigns/{}/ideas/0/100000", challenge_id);
    client.get(path)?.json()
}

pub fn get_funnels_data_for_fund(client: &impl HttpClient) -> Result<Vec<Funnel>, Report> {
    client.get("funnels")?.json()
}
