pub mod db;
pub mod server;

use assert_fs::TempDir;
use lazy_static::lazy_static;
use rand::Rng;
use std::env;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use self::{db::DbBuilder, server::ServerBootstrapper};
use crate::common::data::ArbitrarySnapshotGenerator;
use crate::common::{data, server::Server};
use data::Snapshot;
use server::ServerBootstrapperError;

pub fn get_exe() -> PathBuf {
    const VIT_BIN_NAME: &str = env!("VIT_BIN_NAME");
    let mut path = get_working_directory();
    path.push(VIT_BIN_NAME);
    if cfg!(windows) {
        path.set_extension("exe");
    }
    assert!(
        path.is_file(),
        "File does not exist: {:?}, pwd: {:?}",
        path,
        env::current_dir()
    );
    path
}

pub fn get_cli_exe() -> PathBuf {
    const VIT_BIN_NAME: &str = env!("VIT_CLI_NAME");
    let mut path = get_working_directory();
    path.push(VIT_BIN_NAME);
    if cfg!(windows) {
        path.set_extension("exe");
    }
    assert!(
        path.is_file(),
        "File does not exist: {:?}, pwd: {:?}",
        path,
        env::current_dir()
    );
    path
}

/// Gets working directory
/// Uses std::env::current_exe() for this purpose.
/// Current exe directory is ./target/{profile}/deps/{app_name}.exe
/// Function returns ./target/{profile}
fn get_working_directory() -> PathBuf {
    let mut output_directory: PathBuf = std::env::current_exe().unwrap();

    output_directory.pop();

    if output_directory.ends_with("deps") {
        output_directory.pop();
    }
    output_directory
}

lazy_static! {
    static ref NEXT_AVAILABLE_PORT_NUMBER: AtomicU32 = {
        let initial_port = rand::thread_rng().gen_range(6000, 10999);
        AtomicU32::new(initial_port)
    };
}

pub fn get_available_port() -> u32 {
    NEXT_AVAILABLE_PORT_NUMBER.fetch_add(1, Ordering::SeqCst)
}

pub fn quick_start(temp_dir: &TempDir) -> Result<(Server, Snapshot), ServerBootstrapperError> {
    let snapshot = ArbitrarySnapshotGenerator::default().snapshot();

    let db_path = DbBuilder::new().with_snapshot(&snapshot).build(temp_dir)?;

    let server = ServerBootstrapper::new()
        .with_db_path(db_path.to_str().unwrap())
        .start(temp_dir)?;

    if !server.is_up(&snapshot.token_hash()) {
        return Err(ServerBootstrapperError::FailToBootstrap);
    }

    Ok((server, snapshot))
}

pub fn empty_db(temp_dir: &TempDir) -> PathBuf {
    DbBuilder::new().build(temp_dir).unwrap()
}
