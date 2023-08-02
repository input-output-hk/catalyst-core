use crate::Vote;
use chain_impl_mockchain::vote::Choice;
use chain_impl_mockchain::vote::{PayloadType, TallyResult as TallyResultLib, Weight};
use jormungandr_lib::crypto::hash::Hash;
use jormungandr_lib::interfaces::Block0Configuration;
use jormungandr_lib::interfaces::Initial;
use jormungandr_lib::interfaces::PrivateTallyState;
use jormungandr_lib::interfaces::VoteProposalStatus;
use jormungandr_lib::interfaces::{Tally, VotePlanStatus};
use std::path::Path;
use std::str::FromStr;
use vit_servicing_station_lib::db::models::proposals::FullProposalInfo;

pub trait VotePlanStatusExtension {
    fn dump<P: AsRef<Path>>(&self, file_path: P) -> Result<(), std::io::Error>;
}

impl VotePlanStatusExtension for Vec<VotePlanStatus> {
    fn dump<P: AsRef<Path>>(&self, file_path: P) -> Result<(), std::io::Error> {
        std::fs::write(file_path.as_ref(), serde_json::to_string(&self).unwrap())
    }
}

pub trait VotePlanStatusProvider {
    fn empty_vote_plan_statuses(&self) -> Vec<VotePlanStatus>;
    fn vote_plan_statuses(&self, votes: Vec<CastedVote>) -> Vec<VotePlanStatus>;
}

impl VotePlanStatusProvider for Block0Configuration {
    fn vote_plan_statuses(&self, votes: Vec<CastedVote>) -> Vec<VotePlanStatus> {
        let mut active_vote_plan = self.empty_vote_plan_statuses();

        for vote in votes {
            for vote_plan in active_vote_plan.iter_mut() {
                if let Some(proposal) = vote_plan
                    .proposals
                    .iter_mut()
                    .find(|p| p.proposal_id == vote.proposal_id)
                {
                    let new_tally = match &proposal.tally {
                        Tally::Public { result } => {
                            let mut chain_result: TallyResultLib = result.clone().into();
                            chain_result.add_vote(vote.choice(), vote.weight).unwrap();
                            Tally::Public {
                                result: chain_result.into(),
                            }
                        }
                        Tally::Private { state } => {
                            if let PrivateTallyState::Decrypted { result } = state {
                                let mut chain_result: TallyResultLib = result.clone().into();
                                chain_result.add_vote(vote.choice(), vote.weight).unwrap();
                                Tally::Private {
                                    state: PrivateTallyState::Decrypted {
                                        result: chain_result.into(),
                                    },
                                }
                            } else {
                                panic!("private tally should be already decrypted")
                            }
                        }
                    };
                    proposal.tally = new_tally;
                    proposal.votes_cast += 1;
                }
            }
        }
        active_vote_plan
    }

    fn empty_vote_plan_statuses(&self) -> Vec<VotePlanStatus> {
        let mut active_vote_plan = vec![];
        for initial in &self.initial {
            if let Initial::Cert(cert) = initial {
                if let chain_impl_mockchain::certificate::Certificate::VotePlan(v) =
                    cert.clone().strip_auth().0
                {
                    active_vote_plan.push(VotePlanStatus {
                        id: v.to_id().into(),
                        vote_start: v.vote_start().into(),
                        vote_end: v.vote_end().into(),
                        committee_end: v.committee_end().into(),
                        payload: v.payload_type(),
                        voting_token: v.voting_token().clone().into(),
                        committee_member_keys: v.committee_public_keys().into(),
                        proposals: v
                            .proposals()
                            .iter()
                            .enumerate()
                            .map(|(idx, p)| VoteProposalStatus {
                                index: idx as u8,
                                proposal_id: Hash::from_str(&p.external_id().to_string()).unwrap(),
                                options: p.options().choice_range().clone(),
                                tally: match v.payload_type() {
                                    PayloadType::Public => Tally::Public {
                                        result: TallyResultLib::new(p.options().clone()).into(),
                                    },
                                    PayloadType::Private => Tally::Private {
                                        state: PrivateTallyState::Decrypted {
                                            result: TallyResultLib::new(p.options().clone()).into(),
                                        },
                                    },
                                },
                                votes_cast: 0,
                            })
                            .collect(),
                    });
                }
            }
        }
        active_vote_plan
    }
}

pub struct CastedVote {
    proposal_id: Hash,
    option: Vote,
    weight: Weight,
}

impl CastedVote {
    pub fn from_proposal(proposal: &FullProposalInfo, option: Vote, weight: u64) -> Self {
        CastedVote::from_vec(&proposal.proposal.chain_proposal_id, option, weight)
    }

    pub fn from_vec(proposal_id: &[u8], option: Vote, weight: u64) -> Self {
        Self::new(
            Hash::from_hex(std::str::from_utf8(proposal_id).unwrap()).unwrap(),
            option,
            weight,
        )
    }

    pub fn new(proposal_id: Hash, option: Vote, weight: u64) -> Self {
        Self {
            proposal_id,
            option,
            weight: weight.into(),
        }
    }

    pub fn choice(&self) -> Choice {
        Choice::new(self.option as u8)
    }
}
