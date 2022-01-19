mod tree;

use crate::builders::utils::io::encode_block0;
use crate::builders::utils::io::read_genesis_yaml;
use crate::builders::utils::io::write_genesis_yaml;
use crate::vit_station::DbGenerator;
use crate::Result;
use jormungandr_lib::interfaces::ConsensusLeaderId;
pub use tree::DeploymentTree;
use vit_servicing_station_tests::common::data::ValidVotePlanParameters;
use vit_servicing_station_tests::common::data::{
    ArbitraryValidVotingTemplateGenerator, ExternalValidVotingTemplateGenerator,
};

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

pub fn add_leaders_ids(
    tree: &DeploymentTree,
    consensus_leader_ids: Vec<ConsensusLeaderId>,
) -> Result<()> {
    let mut block0_configuration = read_genesis_yaml(&tree.genesis_path())?;

    if !consensus_leader_ids.is_empty() {
        block0_configuration
            .blockchain_configuration
            .consensus_leader_ids = consensus_leader_ids;
    }
    write_genesis_yaml(block0_configuration, &tree.genesis_path())?;
    encode_block0(&tree.genesis_path(), &tree.block0_path())?;
    Ok(())
}
