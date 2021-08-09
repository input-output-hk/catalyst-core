use crate::common::{
    data,
    startup::{db::DbBuilder, server::ServerBootstrapper},
};
use assert_fs::TempDir;

#[test]
pub fn non_existing_block0_file() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().unwrap();
    let snapshot = data::ArbitrarySnapshotGenerator::default().snapshot();
    let db_path = DbBuilder::new().with_snapshot(&snapshot).build(&temp_dir)?;

    let server = ServerBootstrapper::new()
        .with_db_path(db_path.to_str().unwrap())
        .start(&temp_dir)?;

    assert!(server.is_up(&snapshot.any_token().0));
    Ok(())
}

#[test]
pub fn malformed_path() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().unwrap();
    let snapshot = data::ArbitrarySnapshotGenerator::default().snapshot();
    let db_path = DbBuilder::new().with_snapshot(&snapshot).build(&temp_dir)?;

    let server = ServerBootstrapper::new()
        .with_db_path(db_path.to_str().unwrap())
        .with_block0_path("C:/tmp/a:/block0.bin")
        .start(&temp_dir)?;

    assert!(server.is_up(&snapshot.any_token().0));
    Ok(())
}

#[test]
#[cfg(not(windows))]
pub fn network_path() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().unwrap();
    let snapshot = data::ArbitrarySnapshotGenerator::default().snapshot();
    let db_path = DbBuilder::new().with_snapshot(&snapshot).build(&temp_dir)?;

    let server = ServerBootstrapper::new()
        .with_db_path(db_path.to_str().unwrap())
        .with_block0_path("//tmp/block0.bin")
        .start(&temp_dir)?;

    assert!(server.is_up(&snapshot.any_token().0));
    Ok(())
}
