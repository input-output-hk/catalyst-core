use crate::ideascale::models::de::{Fund, Funnel, Proposal, Stage};

use once_cell::sync::Lazy;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use url::Url;

use std::collections::HashMap;
use std::convert::TryInto;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    RequestError(#[from] reqwest::Error),

    #[error("Could not get value from json, missing attribute {attribute_name}")]
    MissingAttribute { attribute_name: &'static str },
}

#[derive(Debug, Deserialize)]
struct Score {
    #[serde(alias = "ideaId")]
    id: u32,
    #[serde(alias = "avgScoreOfIdea")]
    score: f32,
}

pub type Scores = HashMap<u32, f32>;

static BASE_IDEASCALE_URL: Lazy<url::Url> = Lazy::new(|| {
    "https://cardano.ideascale.com/a/rest/v1/"
        .try_into()
        .unwrap()
});

static CLIENT: Lazy<reqwest::Client> = Lazy::new(reqwest::Client::new);

async fn request_data<T: DeserializeOwned>(api_token: String, url: Url) -> Result<T, Error> {
    CLIENT
        .get(url)
        .header("api_token", api_token)
        .send()
        .await?
        .json()
        .await
        .map_err(Error::RequestError)
}

pub async fn get_funds_data(api_token: String) -> Result<Vec<Fund>, Error> {
    request_data(
        api_token,
        BASE_IDEASCALE_URL.join("campaigns/groups").unwrap(),
    )
    .await
}

pub async fn get_stages(api_token: String) -> Result<Vec<Stage>, Error> {
    request_data(api_token, BASE_IDEASCALE_URL.join("stages").unwrap()).await
}

pub async fn get_assessments_score(assessment_id: u32, api_token: String) -> Result<Scores, Error> {
    let scores: Vec<Score> = request_data(
        api_token,
        BASE_IDEASCALE_URL
            .join(&format!("assessment/{}/results", assessment_id))
            .unwrap(),
    )
    .await?;
    Ok(scores.into_iter().map(|s| (s.id, s.score)).collect())
}

pub async fn get_proposals_data(
    challenge_id: u32,
    api_token: String,
) -> Result<Vec<Proposal>, Error> {
    request_data(
        api_token,
        BASE_IDEASCALE_URL
            // ideascale API have some pager system which is not easy to find in the documentation
            // https://a.ideascale.com/api-docs/index.html#/rest-api-controller-v-1/ideasByCampaignUsingGET_2
            // in this case we want all of them, easiest way is to max out the page size.
            .join(&format!("campaigns/{}/ideas/0/100000", challenge_id))
            .unwrap(),
    )
    .await
}

pub async fn get_funnels_data_for_fund(api_token: String) -> Result<Vec<Funnel>, Error> {
    let challenges: Vec<Funnel> =
        request_data(api_token, BASE_IDEASCALE_URL.join("funnels").unwrap()).await?;
    Ok(challenges)
}
