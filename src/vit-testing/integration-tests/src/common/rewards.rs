use crate::common::static_data::SnapshotExtensions;
use crate::common::vote_plan_status::VotePlanStatusExtension;
use crate::common::vote_plan_status::VotePlanStatusProvider;
use crate::common::CastedVote;
use crate::Vote;
use assert_cmd::assert::OutputAssertExt;
use assert_fs::TempDir;
use catalyst_toolbox::testing::ProposerRewardsCommand;
use chain_addr::{Address, AddressReadable, Discrimination, Kind};
use jormungandr_automation::testing::block0;
use jormungandr_lib::crypto::key::Identifier;
use jortestkit::prelude::{enhance_exe_name, find_exec};
use serde::Deserialize;
use std::fs::File;
use std::path::{Path, PathBuf};
use thiserror::Error;
use vit_servicing_station_lib::db::models::proposals::FullProposalInfo;
use vit_servicing_station_tests::common::data::Snapshot;
use vitup::builders::utils::DeploymentTree;

pub type VotesRegistry = Vec<(FullProposalInfo, Vec<(Vote, u64)>)>;

pub fn funded_proposals(
    testing_directory: &TempDir,
    snapshot: &Snapshot,
    registry: VotesRegistry,
) -> Result<ProposerRewardsResult, Error> {
    let deployment = DeploymentTree::from(testing_directory);
    let block0_configuration = block0::get_block(
        deployment
            .block0_path()
            .to_str()
            .ok_or(Error::InvalidBlock0Path)?,
    )?;

    let proposals_json = testing_directory.path().join("proposals.json");
    let challenges_json = testing_directory.path().join("challenges.json");
    snapshot.dump_proposals(&proposals_json)?;
    snapshot.dump_challenges(&challenges_json)?;

    let votes = registry
        .iter()
        .flat_map(|(proposal, votes)| {
            votes
                .iter()
                .map(|vote| CastedVote::from_proposal(proposal, vote.0, vote.1))
                .collect::<Vec<CastedVote>>()
        })
        .collect();

    let active_vote_plan = block0_configuration.vote_plan_statuses(votes);
    let discrimination = block0_configuration.blockchain_configuration.discrimination;
    let prefix = match discrimination {
        Discrimination::Test => "ta",
        Discrimination::Production => "ca",
    };
    let committee_addresses: Vec<String> = block0_configuration
        .blockchain_configuration
        .committees
        .iter()
        .map(|x| {
            let key = Identifier::from_hex(&x.to_hex()).unwrap();
            let address = AddressReadable::from_address(
                prefix,
                &Address(discrimination, Kind::Account(key.into_public_key())),
            );
            address.to_string()
        })
        .collect();

    let vote_plan_json = testing_directory.path().join("vote_plan.json");
    active_vote_plan.dump(&vote_plan_json)?;
    let output = testing_directory.path().join("rewards.csv");

    let committee_yaml = testing_directory.path().join("committee.yaml");
    std::fs::write(
        &committee_yaml,
        serde_json::to_string(&committee_addresses)?,
    )?;

    let mut rewards_command = ProposerRewardsCommand::default()
        .python_exec(find_python_exec())
        .output_file(output.clone())
        .block0_path(deployment.genesis_path())
        .total_stake_threshold(0.01)
        .approval_threshold(0.05)
        .output_format("csv".to_string())
        .proposals_path(
            proposals_json
                .to_str()
                .ok_or(Error::InvalidProposalJsonPath)?
                .to_string(),
        )
        .active_voteplan_path(
            vote_plan_json
                .to_str()
                .ok_or(Error::InvalidVotePlanJsonPath)?
                .to_string(),
        )
        .committee_keys_path(
            committee_yaml
                .to_str()
                .ok_or(Error::InvalidCommitteeKeysPath)?
                .to_string(),
        )
        .challenges_path(
            challenges_json
                .to_str()
                .ok_or(Error::InvalidChallengesJsonPath)?
                .to_string(),
        )
        .cmd(testing_directory)?;

    rewards_command.assert().success();
    Ok(ProposerRewardsResult::from(output))
}

pub fn find_python_exec() -> PathBuf {
    let proposals = ["python", "python3", "python3.8", "python3.10"];
    for proposal in proposals {
        if let Some(path) = find_exec(enhance_exe_name(Path::new(proposal))) {
            return path;
        }
    }
    panic!("cannot find python executable. Tried: {:?}", proposals);
}

pub struct ProposerRewards(Vec<ProposerReward>);

impl From<Vec<ProposerReward>> for ProposerRewards {
    fn from(proposer_rewards: Vec<ProposerReward>) -> Self {
        Self(proposer_rewards)
    }
}

impl ProposerRewards {
    pub fn is_funded<S: Into<String>>(&self, proposal_title: S) -> Result<bool, Error> {
        let proposal_title = proposal_title.into();
        let proposal_record = self
            .0
            .iter()
            .find(|r| r.proposal == proposal_title)
            .ok_or(Error::CannotFindProposal(proposal_title))?;
        Ok(proposal_record.status == "FUNDED")
    }
}

#[derive(Debug, Deserialize)]
pub struct ProposerReward {
    pub internal_id: u32,
    pub proposal_id: String,
    pub proposal: String,
    pub overall_score: f32,
    pub yes: u32,
    pub no: u32,
    pub result: u32,
    pub meets_approval_threshold: String,
    pub requested_dollars: u32,
    pub status: String,
    pub fund_depletion: u32,
    pub not_funded_reason: String,
    pub link_to_ideascale: String,
}

pub struct ProposerRewardsResult {
    template: PathBuf,
}

impl From<PathBuf> for ProposerRewardsResult {
    fn from(template: PathBuf) -> Self {
        Self { template }
    }
}

impl ProposerRewardsResult {
    fn file_path<S: Into<String>>(&self, prefix: S) -> PathBuf {
        let mut output = self.template.clone();
        output.set_file_name(format!(
            "{}_{}.{}",
            self.template.file_stem().unwrap().to_str().unwrap(),
            prefix.into().replace(' ', "_"),
            self.template.extension().unwrap().to_str().unwrap()
        ));
        output
    }

    pub fn challenge_results<S: Into<String>>(
        &self,
        challenge_title: S,
    ) -> Result<ProposerRewards, Error> {
        let file_path = self.file_path(challenge_title);
        let file = File::open(file_path)?;
        let mut rdr = csv::Reader::from_reader(file);
        let mut records = Vec::new();
        for result in rdr.deserialize() {
            records.push(result?);
        }
        Ok(records.into())
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid block0 path")]
    InvalidBlock0Path,
    #[error(transparent)]
    Block0(#[from] jormungandr_automation::testing::block0::Block0Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error("invalid proposal json path")]
    InvalidProposalJsonPath,
    #[error("invalid vote plan json path")]
    InvalidVotePlanJsonPath,
    #[error("invalid commitee keys path")]
    InvalidCommitteeKeysPath,
    #[error("invalid challenges json path")]
    InvalidChallengesJsonPath,
    #[error(transparent)]
    ProposerRewardsCmd(#[from] catalyst_toolbox::testing::Error),
    #[error(transparent)]
    Csv(#[from] csv::Error),
    #[error("cannot find proposal entry: {0}")]
    CannotFindProposal(String),
}
