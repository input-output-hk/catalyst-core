pub mod archive;
pub mod console;
pub mod csv;
pub mod env;
pub mod file;
pub mod github;
pub mod load;
pub mod measurement;
pub mod openssl;
pub mod predicates;
pub mod process;
pub mod string;
pub mod web;

#[macro_use(lazy_static)]
extern crate lazy_static;

pub mod prelude {
    pub use crate::archive::decompress;
    pub use crate::console::*;
    pub use crate::csv::CsvFileBuilder;
    pub use crate::env::*;
    pub use crate::file::*;
    pub use crate::github::{GitHubApi, GitHubApiError, Release};
    pub use crate::load;
    pub use crate::measurement::{
        benchmark_consumption, benchmark_efficiency, benchmark_endurance, benchmark_speed,
        ConsumptionBenchmarkError, ConsumptionBenchmarkRun, EfficiencyBenchmarkDef,
        EfficiencyBenchmarkFinish, EfficiencyBenchmarkRun, Endurance, EnduranceBenchmarkDef,
        EnduranceBenchmarkFinish, EnduranceBenchmarkRun, NamedProcess, ResourcesUsage, Speed,
        SpeedBenchmarkDef, SpeedBenchmarkFinish, SpeedBenchmarkRun, Thresholds, Timestamp,
    };
    pub use crate::openssl::{generate_keys, Openssl};
    pub use crate::predicates::*;
    pub use crate::process::{
        self, output_extensions::ProcessOutput, ProcessError, Wait, WaitBuilder,
    };
    pub use crate::web::download_file;
}
