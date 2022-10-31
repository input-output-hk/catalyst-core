/// Snapshot reference tests which perform output comparison for historical snapshots
#[cfg(test)]
#[cfg(feature = "reference_tests")]
mod reference;

/// E2E tests for integrations between voting tool and db sync database
#[cfg(test)]
#[cfg(feature = "e2e_tests")]
mod e2e;

/// Functional tests for voting tools and db sync mock
#[cfg(test)]
mod functional;

/// Test api for internal tests as well for building mocks and varius testing utils
/// outside the project
#[cfg(any(test, feature = "test_api"))]
pub mod test_api;
