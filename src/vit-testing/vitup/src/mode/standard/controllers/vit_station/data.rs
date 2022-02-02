use super::{
    ArbitraryValidVotingTemplateGenerator, DbBuilder, ExternalValidVotingTemplateGenerator,
    ValidVotePlanGenerator, ValidVotePlanParameters, ValidVotingTemplateGenerator,
};
use crate::builders::utils::DeploymentTree;
use assert_fs::TempDir;
use std::path::{Path, PathBuf};

pub struct DbGenerator {
    parameters: ValidVotePlanParameters,
    migration_scripts_path: PathBuf,
}

impl DbGenerator {
    pub fn new(
        parameters: ValidVotePlanParameters,
        migration_scripts_path: Option<PathBuf>,
    ) -> Self {
        Self {
            parameters,
            migration_scripts_path: migration_scripts_path.unwrap_or_else(|| {
                std::path::Path::new("../").join("resources/vit_station/migration")
            }),
        }
    }

    pub fn build(self, db_file: &Path, template_generator: &mut dyn ValidVotingTemplateGenerator) {
        std::fs::File::create(&db_file).unwrap();

        let mut generator = ValidVotePlanGenerator::new(self.parameters);
        let snapshot = generator.build(template_generator);

        let temp_dir = TempDir::new().unwrap().into_persistent();
        let temp_db_path = DbBuilder::new()
            .with_snapshot(&snapshot)
            .with_migrations_from(self.migration_scripts_path)
            .build(&temp_dir)
            .unwrap();

        jortestkit::file::copy_file(temp_db_path, db_file, true);
    }
}

pub fn generate_random_database(tree: &DeploymentTree, vit_parameters: ValidVotePlanParameters) {
    let mut template_generator = ArbitraryValidVotingTemplateGenerator::new();
    DbGenerator::new(vit_parameters, None).build(&tree.database_path(), &mut template_generator);
}

pub fn generate_database(
    tree: &DeploymentTree,
    vit_parameters: ValidVotePlanParameters,
    mut template_generator: ExternalValidVotingTemplateGenerator,
) {
    DbGenerator::new(vit_parameters, None).build(&tree.database_path(), &mut template_generator);
}
