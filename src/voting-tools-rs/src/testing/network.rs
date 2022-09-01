use std::time::Duration;

use reqwest::{
    blocking::Client,
    header::{HeaderName, HeaderValue, CONTENT_TYPE},
};
use serde::Serialize;

use crate::model::{Output, SlotNo};

fn api_token() -> HeaderValue {
    std::env::var("API_KEY").expect("$API_KEY should be set to a valid API key for https://snapshot-trigger-service-testnet.gov.iog.io/").try_into().expect("$API_KEY contained invalid bytes")
}

const BASE_URL: &str = "https://snapshot-trigger-service-testnet.gov.iog.io/api";

static API_TOKEN: HeaderName = HeaderName::from_static("api-token");
static JSON: HeaderValue = HeaderValue::from_static("application/json");

pub fn get_from_network(slot_no: Option<SlotNo>) -> Vec<Output> {
    let client = make_client();
    let job = create_job(&client, Some(123.into()));

    while !is_ready(&client, &job) {
        std::thread::sleep(Duration::from_secs(1));
    }

    get(&client, &job)
}

fn make_client() -> Client {
    let headers = [
        (API_TOKEN.clone(), api_token()),
        (CONTENT_TYPE.clone(), JSON.clone()),
    ];

    Client::builder()
        .default_headers(headers.into_iter().collect())
        .build()
        .unwrap()
}

fn create_job(client: &Client, slot_no: Option<SlotNo>) -> String {
    #[derive(Serialize)]
    struct CreateJob {
        threshold: u64,
        slot_no: Option<SlotNo>,
    }

    let job = CreateJob {
        slot_no,
        threshold: 0,
    };

    client
        .post(format!("{BASE_URL}/job/new"))
        .json(&job)
        .send()
        .unwrap()
        .json()
        .unwrap()
}

fn is_ready(client: &Client, job: &str) -> bool {
    let response = client
        .get(format!("{BASE_URL}/job/status/{job}"))
        .send()
        .unwrap()
        .text()
        .unwrap();

    println!("res: {response}");

    response == "ready"
}

fn get(client: &Client, job: &str) -> Vec<Output> {
    client
        .get(format!("{BASE_URL}/job/files/get/{job}/snapshot.json"))
        .send()
        .unwrap()
        .json()
        .unwrap()
}

