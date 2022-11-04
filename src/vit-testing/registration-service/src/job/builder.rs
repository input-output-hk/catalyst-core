use crate::config::NetworkType;
pub use info::JobOutputInfo;
use std::path::{Path, PathBuf};

use super::info;
use crate::cardano::cli::CardanoCli;
use crate::catalyst_toolbox::CatalystToolboxCli;
use crate::config::Configuration;
use crate::job::VoteRegistrationJob;
use crate::VoterRegistrationCli;

pub struct VoteRegistrationJobBuilder {
    jcli: PathBuf,
    cardano_cli: CardanoCli,
    voter_registration: VoterRegistrationCli,
    catalyst_toolbox: CatalystToolboxCli,
    network: NetworkType,
    working_dir: PathBuf,
}

impl VoteRegistrationJobBuilder {
    pub fn new(config: Configuration) -> Self {
        Self {
            jcli: config.jcli,
            cardano_cli: CardanoCli::new(config.cardano_cli),
            voter_registration: VoterRegistrationCli::new(config.voter_registration),
            catalyst_toolbox: CatalystToolboxCli::new(config.catalyst_toolbox),
            network: NetworkType::Mainnet,
            working_dir: config.inner.result_dir,
        }
    }

    pub fn with_jcli<P: AsRef<Path>>(mut self, jcli: P) -> Self {
        self.jcli = jcli.as_ref().to_path_buf();
        self
    }

    pub fn with_cardano_cli(mut self, cardano_cli: CardanoCli) -> Self {
        self.cardano_cli = cardano_cli;
        self
    }

    pub fn with_voter_registration<P: AsRef<Path>>(
        mut self,
        voter_registration: VoterRegistrationCli,
    ) -> Self {
        self.voter_registration = voter_registration;
        self
    }

    pub fn with_catalyst_toolbox<P: AsRef<Path>>(
        mut self,
        catalyst_toolbox: CatalystToolboxCli,
    ) -> Self {
        self.catalyst_toolbox = catalyst_toolbox;
        self
    }

    pub fn with_network(mut self, network: NetworkType) -> Self {
        self.network = network;
        self
    }

    pub fn with_working_dir<P: AsRef<Path>>(mut self, working_dir: P) -> Self {
        self.working_dir = working_dir.as_ref().to_path_buf();
        self
    }

    pub fn build(self) -> VoteRegistrationJob {
        VoteRegistrationJob {
            jcli: self.jcli,
            cardano_cli: self.cardano_cli,
            voter_registration: self.voter_registration,
            catalyst_toolbox: self.catalyst_toolbox,
            network: self.network,
        }
    }
}
