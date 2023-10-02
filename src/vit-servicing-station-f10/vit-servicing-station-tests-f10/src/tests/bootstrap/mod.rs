use crate::common::{
    data,
    startup::{db::DbBuilder, server::ServerBootstrapper},
};
use assert_fs::TempDir;

pub mod arguments;

#[test]
pub fn bootstrap_with_random_data() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().unwrap().into_persistent();
    let snapshot = data::ArbitrarySnapshotGenerator::default().snapshot();
    let db_path = DbBuilder::new().with_snapshot(&snapshot).build(&temp_dir)?;

    let server = ServerBootstrapper::new()
        .with_db_path(db_path.to_str().unwrap())
        .start(&temp_dir)?;

    std::thread::sleep(std::time::Duration::from_secs(1));
    assert!(server.is_up(&snapshot.any_token().0));
    Ok(())
}
