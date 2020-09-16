use crate::common::cli::{VitCli, VitCliCommand};
use crate::common::startup::quick_start;
use assert_cmd::assert::OutputAssertExt;
use assert_fs::{fixture::PathChild, TempDir};
use hyper::StatusCode;
use jortestkit::process::output_extensions::ProcessOutput;
use std::error::Error;

#[test]
pub fn generate_token() {
    let vit_cli: VitCliCommand = Default::default();
    let output = vit_cli
        .api_token()
        .generate()
        .n(2)
        .build()
        .assert()
        .success()
        .get_output()
        .as_multi_line();

    assert_eq!(2, output.len());

    for line in output {
        assert_eq!(14, line.len())
    }
}

#[test]
pub fn generate_token_for_given_size_and_n() {
    let vit_cli: VitCliCommand = Default::default();
    let output = vit_cli
        .api_token()
        .generate()
        .n(3)
        .size(15)
        .build()
        .assert()
        .success()
        .get_output()
        .as_multi_line();

    assert_eq!(3, output.len());

    for line in output {
        assert_eq!(20, line.len())
    }
}

#[test]
pub fn add_generated_token_to_db() -> Result<(), Box<dyn Error>> {
    let temp_dir = TempDir::new().unwrap();
    let (server, _snapshot) = quick_start(&temp_dir).unwrap();

    let vit_cli: VitCli = Default::default();
    let tokens = vit_cli.generate_tokens(1);

    let vit_cli: VitCliCommand = Default::default();
    vit_cli
        .api_token()
        .add()
        .db_url(server.settings().db_url.clone())
        .tokens(&tokens)
        .build()
        .assert()
        .success();

    let first_token = tokens.get(0).unwrap();

    assert_eq!(
        server
            .rest_client_with_token(first_token)
            .health_raw()?
            .status(),
        StatusCode::OK
    );
    Ok(())
}

#[test]
pub fn add_generated_token_to_db_negative() -> Result<(), Box<dyn Error>> {
    let temp_dir = TempDir::new().unwrap().into_persistent();
    let (server, _snapshot) = quick_start(&temp_dir).unwrap();

    let vit_cli: VitCli = Default::default();
    let tokens = vit_cli.generate_tokens(1);

    let vit_cli: VitCliCommand = Default::default();
    vit_cli
        .api_token()
        .add()
        .db_url(temp_dir.child("fake.db").path().to_str().unwrap())
        .tokens(&tokens)
        .build()
        .assert()
        .failure();

    let vit_cli: VitCliCommand = Default::default();
    vit_cli
        .api_token()
        .add()
        .db_url(server.settings().db_url.clone())
        .tokens_as_str("some_random_token")
        .build()
        .assert()
        .failure();

    let vit_cli: VitCliCommand = Default::default();
    vit_cli
        .api_token()
        .add()
        .db_url(server.settings().db_url.clone())
        .tokens_as_str("randomtoken1;randomtoken2")
        .build()
        .assert()
        .failure();
    Ok(())
}
