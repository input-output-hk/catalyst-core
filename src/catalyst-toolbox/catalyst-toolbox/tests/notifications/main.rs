mod verifier;

use crate::verifier::NotificationsVerifier;
use assert_cmd::assert::OutputAssertExt;
use assert_cmd::cargo::CommandCargoExt;
use assert_fs::fixture::PathChild;
use assert_fs::TempDir;
use catalyst_toolbox::notifications::responses::create_message::CreateMessageResponse;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

#[test]
pub fn sanity_notification() {
    let access_token = get_env("NOTIFICATION_ACCESS_TOKEN");
    let app_token = get_env("NOTIFICATION_APP_CODE");
    let message = "hello";

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.child("notification.msg");

    create_message_file(file_path.path(), message);

    let result = Command::cargo_bin("catalyst-toolbox")
        .unwrap()
        .arg("push")
        .arg("send")
        .arg("from-args")
        .arg("--access-token")
        .arg(&access_token)
        .arg("--application")
        .arg(&app_token)
        .arg(file_path.path())
        .assert()
        .success();

    let output = std::str::from_utf8(&result.get_output().stdout).unwrap();
    let response: CreateMessageResponse = serde_json::from_str(output).unwrap();
    println!("{:?}", response);
    let id = response.response.messages.get(0).unwrap();
    NotificationsVerifier::new(&access_token)
        .verify_message_done_with_text(id, &message.to_string());
}

fn get_env<S: Into<String>>(env_name: S) -> String {
    let env_name = env_name.into();
    std::env::var(&env_name).unwrap_or_else(|_| panic!("{} not defined", env_name))
}

fn create_message_file<P: AsRef<Path>, S: Into<String>>(path: P, message: S) {
    let mut file = File::create(path.as_ref()).unwrap();
    let message = format!("\"{}\"", message.into());
    file.write_all(message.as_bytes()).unwrap();
}
