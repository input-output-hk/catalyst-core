use super::{
    ArbitraryValidVotingTemplateGenerator, DbBuilder, ExternalValidVotingTemplateGenerator,
    ValidVotePlanGenerator, ValidVotePlanParameters, ValidVotingTemplateGenerator,
};
use crate::builders::utils::DeploymentTree;
use crate::config::MigrationError;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    DbBuilder(#[from] vit_servicing_station_tests::common::startup::db::DbBuilderError),
    #[error(transparent)]
    Migration(#[from] MigrationError),
}

pub struct DbGenerator {
    parameters: ValidVotePlanParameters,
}

impl DbGenerator {
    pub fn new(parameters: ValidVotePlanParameters) -> Self {
        Self { parameters }
    }

    pub fn build(
        self,
        db_file: &Path,
        template_generator: &mut dyn ValidVotingTemplateGenerator,
    ) -> Result<(), Error> {
        std::fs::File::create(db_file)?;

        let mut generator = ValidVotePlanGenerator::new(self.parameters);
        let snapshot = generator.build(template_generator);
        DbBuilder::new()
            .with_snapshot(&snapshot)
            .build_into_path(db_file)
            .map(|_| ())
            .map_err(Into::into)
    }
}

pub fn generate_random_database(
    tree: &DeploymentTree,
    vit_parameters: ValidVotePlanParameters,
) -> Result<(), Error> {
    let mut template_generator = ArbitraryValidVotingTemplateGenerator::new();
    DbGenerator::new(vit_parameters).build(&tree.database_path(), &mut template_generator)
}

pub fn generate_database(
    tree: &DeploymentTree,
    vit_parameters: ValidVotePlanParameters,
    mut template_generator: ExternalValidVotingTemplateGenerator,
) -> Result<(), Error> {
    DbGenerator::new(vit_parameters).build(&tree.database_path(), &mut template_generator)
}
