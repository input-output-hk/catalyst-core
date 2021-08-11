use assert_fs::TempDir;
use std::path::Path;
use vit_servicing_station_tests::common::data::ValidVotePlanParameters;
use vit_servicing_station_tests::common::data::{
    ValidVotePlanGenerator, ValidVotingTemplateGenerator,
};
use vit_servicing_station_tests::common::startup::db::DbBuilder;
pub struct DbGenerator {
    parameters: ValidVotePlanParameters,
}

impl DbGenerator {
    pub fn new(parameters: ValidVotePlanParameters) -> Self {
        Self { parameters }
    }

    pub fn build(self, db_file: &Path, template_generator: &mut dyn ValidVotingTemplateGenerator) {
        std::fs::File::create(&db_file).unwrap();

        let mut generator = ValidVotePlanGenerator::new(self.parameters);
        let snapshot = generator.build(template_generator);

        println!("{:?}", snapshot);

        let path = std::path::Path::new("../").join("resources/vit_station/migration");

        let temp_dir = TempDir::new().unwrap().into_persistent();
        let temp_db_path = DbBuilder::new()
            .with_snapshot(&snapshot)
            .with_migrations_from(path)
            .build(&temp_dir)
            .unwrap();

        jortestkit::file::copy_file(temp_db_path, db_file, true);
    }
}
