use std::path::Path;
use vit_servicing_station_tests::common::data::Snapshot;

pub trait SnapshotExtensions {
    fn dump_proposals<P: AsRef<Path>>(&self, file_path: P) -> Result<(), std::io::Error>;
    fn dump_challenges<P: AsRef<Path>>(&self, file_path: P) -> Result<(), std::io::Error>;
}

impl SnapshotExtensions for Snapshot {
    fn dump_proposals<P: AsRef<Path>>(&self, file_path: P) -> Result<(), std::io::Error> {
        std::fs::write(
            file_path.as_ref(),
            serde_json::to_string(&self.proposals()).unwrap(),
        )
    }

    fn dump_challenges<P: AsRef<Path>>(&self, file_path: P) -> Result<(), std::io::Error> {
        std::fs::write(
            file_path.as_ref(),
            serde_json::to_string(&self.challenges()).unwrap(),
        )
    }
}
