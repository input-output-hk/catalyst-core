use jortestkit::prelude::WaitBuilder;
use registration_service::{
    client::rest::RegistrationRestClient, context::State, request::Request,
};
use snapshot_trigger_service::{client::rest::SnapshotRestClient, config::JobParameters};

#[test]
pub fn e2e_flow_using_voter_registration_local_vitup_and_iapyx() {
    let registration_token = std::env::var("REGISTRATION_TOKEN")
        .unwrap_or_else(|_| "REGISTRATION_TOKEN not defined".to_owned());
    let registration_address = std::env::var("REGISTRATION_ADDRESS")
        .unwrap_or_else(|_| "REGISTRATION_ADDRESS not defined".to_owned());
    let snapshot_token =
        std::env::var("SNAPSHOT_TOKEN").unwrap_or_else(|_| "SNAPSHOT_TOKEN not defined".to_owned());
    let snapshot_address = std::env::var("SNAPSHOT_ADDRESS")
        .unwrap_or_else(|_| "SNAPSHOT_ADDRESS not defined".to_owned());
    let payment_skey =
        std::env::var("PAYMENT_SKEY").unwrap_or_else(|_| "PAYMENT_SKEY not defined".to_owned());
    let payment_vkey =
        std::env::var("PAYMENT_VKEY").unwrap_or_else(|_| "PAYMENT_VKEY not defined".to_owned());
    let stake_skey =
        std::env::var("STAKE_SKEY").unwrap_or_else(|_| "STAKE_SKEY not defined".to_owned());
    let stake_vkey =
        std::env::var("STAKE_VKEY").unwrap_or_else(|_| "STAKE_VKEY not defined".to_owned());

    let registration_client = RegistrationRestClient::new_with_token(
        registration_token.to_string(),
        registration_address.to_string(),
    );

    let registration_request = Request {
        payment_skey: payment_skey.to_string(),
        payment_vkey: payment_vkey.to_string(),
        stake_skey: stake_skey.to_string(),
        stake_vkey: stake_vkey.to_string(),
    };

    let registration_job_id = registration_client.job_new(registration_request).unwrap();

    let wait = WaitBuilder::new().tries(10).sleep_between_tries(10).build();
    println!("waiting for registration job");
    let registration_jobs_status = registration_client
        .wait_for_job_finish(registration_job_id.clone(), wait)
        .unwrap();
    println!("{:?}", registration_jobs_status);

    registration_client
        .download_qr(registration_job_id.clone(), "C:\\tmp\\qr_1234.png")
        .unwrap();

    let snapshot_client = SnapshotRestClient::new_with_token(
        snapshot_token.to_string(),
        snapshot_address.to_string(),
    );

    let job_param = match registration_jobs_status {
        State::Finished { info, .. } => JobParameters {
            slot_no: Some(info.slot_no),
            threshold: 1_000_000,
        },
        _ => panic!("registration job should be already finished"),
    };

    let snapshot_job_id = snapshot_client.job_new(job_param).unwrap();

    let wait = WaitBuilder::new().tries(10).sleep_between_tries(10).build();

    println!("waiting for snapshot job");
    let snapshot_jobs_status = snapshot_client.wait_for_job_finish(snapshot_job_id.clone(), wait);
    println!("{:?}", snapshot_jobs_status);

    let snapshot_file = snapshot_client.download_snapshot(snapshot_job_id, "C:\\tmp");
}
