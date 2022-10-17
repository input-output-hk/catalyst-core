use crate::interfaces::{
    config_params::FromConfigParamError, BlockDate, ConfigParams, ConsensusLeaderId,
};
use chain_impl_mockchain::{certificate::UpdateProposal, update::UpdateProposalState};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct UpdateProposalStateDef {
    pub proposal: UpdateProposalDef,
    pub proposal_date: BlockDate,
    pub votes: Vec<ConsensusLeaderId>,
}

impl TryFrom<UpdateProposalState> for UpdateProposalStateDef {
    type Error = FromConfigParamError;
    fn try_from(state: UpdateProposalState) -> Result<Self, FromConfigParamError> {
        Ok(UpdateProposalStateDef {
            proposal: state.proposal.try_into()?,
            proposal_date: state.proposal_date.into(),
            votes: state
                .votes
                .into_iter()
                .map(|(x, _)| ConsensusLeaderId::from(x.clone()))
                .collect(),
        })
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UpdateProposalDef {
    pub config_params: ConfigParams,
    pub proposer_id: ConsensusLeaderId,
}

impl TryFrom<UpdateProposal> for UpdateProposalDef {
    type Error = FromConfigParamError;
    fn try_from(update_proposal: UpdateProposal) -> Result<Self, FromConfigParamError> {
        let mut config_params = Vec::new();
        for config_param in update_proposal.changes().iter() {
            config_params.push(config_param.clone().try_into()?);
        }

        Ok(UpdateProposalDef {
            config_params: ConfigParams::new(config_params),
            proposer_id: ConsensusLeaderId::from(update_proposal.proposer_id().clone()),
        })
    }
}
