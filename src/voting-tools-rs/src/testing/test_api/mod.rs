//mod fake;
mod output_assertions;

// pub use fake::MockDbProvider; // Mocking like this is pointless, and pulls in almost the entire catalyst-core repo.
pub use output_assertions::{SnapshotOutputAssert, VerifiableSnapshotOutput};
