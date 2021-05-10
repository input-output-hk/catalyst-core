use jortestkit::prelude::WaitBuilder;
use registration_service::{
    client::rest::RegistrationRestClient, context::State, request::Request,
};
use snapshot_trigger_service::{client::rest::SnapshotRestClient, config::JobParameters};

const REGISTRATION_TOKEN: &str = "I-O3OaHVdD1Www";
const REGISTRATION_ADDRESS: &str = "https://registration-testnet.vit.iohk.io";

const SNAPSHOT_TOKEN: &str = "UarouhaiNgiiwahcaNgai8aexaiTh0me";
const SNAPSHOT_ADDRESS: &str = "https://snapshot-testnet.vit.iohk.io";

const PAYMENT_SKEY: &str = "58205d9e4a747b7115d3fcb36e56308ebe3dd327b109fa95f147ff2f1e423c103848";
const PAYMENT_VKEY: &str = "5820530136f75366d408e643e267e41c1f3aa15f6b017750af3371aabc7feb77d50c";
const STAKE_SKEY: &str = "5820dad960491123979e49f0f00d319ee9e68a3530fe0df3f878ff8f3d19bd0ca696";
const STAKE_VKEY: &str = "5820e542b6a0ced80e1ab5bda70311bf643b9011ee04411737f3e0136825ef47f2d8";

#[test]
pub fn e2e_flow_using_voter_registration_local_vitup_and_iapyx() {
    let registration_client = RegistrationRestClient::new_with_token(
        REGISTRATION_TOKEN.to_string(),
        REGISTRATION_ADDRESS.to_string(),
    );

    let registration_request = Request {
        payment_skey: PAYMENT_SKEY.to_string(),
        payment_vkey: PAYMENT_VKEY.to_string(),
        stake_skey: STAKE_SKEY.to_string(),
        stake_vkey: STAKE_VKEY.to_string(),
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
        SNAPSHOT_TOKEN.to_string(),
        SNAPSHOT_ADDRESS.to_string(),
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
