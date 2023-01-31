use super::{
    ArbitraryValidVotingTemplateGenerator, DbBuilder, ExternalValidVotingTemplateGenerator,
    ValidVotePlanGenerator, ValidVotePlanParameters, ValidVotingTemplateGenerator,
};
use crate::config::MigrationError;
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
        template_generator: &mut dyn ValidVotingTemplateGenerator,
    ) -> Result<String, Error> {
        let mut generator = ValidVotePlanGenerator::new(self.parameters);
        let snapshot = generator.build(template_generator);
        DbBuilder::new()
            .with_snapshot(&snapshot)
            .build()
            .map_err(Into::into)
    }
}

pub fn generate_random_database(vit_parameters: ValidVotePlanParameters) -> Result<String, Error> {
    let mut template_generator = ArbitraryValidVotingTemplateGenerator::new();
    DbGenerator::new(vit_parameters).build(&mut template_generator)
}

pub fn generate_database(
    vit_parameters: ValidVotePlanParameters,
    mut template_generator: ExternalValidVotingTemplateGenerator,
) -> Result<String, Error> {
    DbGenerator::new(vit_parameters).build(&mut template_generator)
}
