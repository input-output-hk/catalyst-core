use super::ExplorerVerifier;
use crate::jormungandr::explorer::data::all_vote_plans::{
    self, AllVotePlansTipAllVotePlansEdgesNodeProposalsTally::*,
    AllVotePlansTipAllVotePlansEdgesNodeProposalsVotesEdgesNodePayload::*, *,
};
use chain_impl_mockchain::{testing::data::Wallet, vote, vote::Choice};
use jormungandr_lib::interfaces::{PrivateTallyState, Tally, VotePlanStatus};
use std::collections::HashMap;

impl ExplorerVerifier {
    pub fn assert_all_vote_plans(
        explorer_vote_plans: Vec<AllVotePlansTipAllVotePlansEdges>,
        vote_plan_statuses: Vec<VotePlanStatus>,
        proposal_votes: HashMap<String, HashMap<String, Vec<(Wallet, Choice)>>>,
    ) {
        let mut matching_vote_plans = 0;
        assert_eq!(vote_plan_statuses.len(), explorer_vote_plans.len());
        for vote_plan_status in &vote_plan_statuses {
            for explorer_vote_plan_edge in &explorer_vote_plans {
                let explorer_vote_plan = &explorer_vote_plan_edge.node;
                if explorer_vote_plan.id == vote_plan_status.id.to_string() {
                    matching_vote_plans += 1;
                    assert_eq!(
                        explorer_vote_plan.vote_start.epoch.id,
                        vote_plan_status.vote_start.epoch().to_string()
                    );
                    assert_eq!(
                        explorer_vote_plan.vote_start.slot,
                        vote_plan_status.vote_start.slot().to_string()
                    );
                    assert_eq!(
                        explorer_vote_plan.vote_end.epoch.id,
                        vote_plan_status.vote_end.epoch().to_string()
                    );
                    assert_eq!(
                        explorer_vote_plan.vote_end.slot,
                        vote_plan_status.vote_end.slot().to_string()
                    );
                    assert_eq!(
                        explorer_vote_plan.committee_end.epoch.id,
                        vote_plan_status.committee_end.epoch().to_string()
                    );
                    assert_eq!(
                        explorer_vote_plan.committee_end.slot,
                        vote_plan_status.committee_end.slot().to_string()
                    );
                    match explorer_vote_plan.payload_type {
                        all_vote_plans::PayloadType::PUBLIC => assert!(matches!(
                            vote_plan_status.payload,
                            vote::PayloadType::Public
                        )),
                        all_vote_plans::PayloadType::PRIVATE => assert!(matches!(
                            vote_plan_status.payload,
                            vote::PayloadType::Private
                        )),
                        all_vote_plans::PayloadType::Other(_) => panic!("Wrong payload type"),
                    }

                    assert_eq!(
                        explorer_vote_plan.proposals.len(),
                        vote_plan_status.proposals.len()
                    );
                    for explorer_proposal in &explorer_vote_plan.proposals {
                        let vote_proposal_status =
                            match vote_plan_status.proposals.iter().position(|proposal| {
                                explorer_proposal.proposal_id == proposal.proposal_id.to_string()
                            }) {
                                Some(index) => vote_plan_status.proposals[index].clone(),
                                None => panic!("Proposal id not found"),
                            };
                        assert_eq!(
                            vote_proposal_status.options.start,
                            explorer_proposal.options.start as u8
                        );
                        assert_eq!(
                            vote_proposal_status.options.end,
                            explorer_proposal.options.end as u8
                        );
                        match &vote_proposal_status.tally {
                            Tally::Public { result } => {
                                assert!(explorer_proposal.tally.is_some());
                                if let TallyPublicStatus(explorer_tally_status) =
                                    explorer_proposal.tally.as_ref().unwrap()
                                {
                                    assert_eq!(
                                        result.results.len(),
                                        explorer_tally_status.results.len()
                                    );
                                    let matching_results = result
                                        .results
                                        .iter()
                                        .zip(explorer_tally_status.results.iter())
                                        .filter(|&(a, b)| &a.to_string() == b)
                                        .count();
                                    assert_eq!(matching_results, result.results.len());
                                    assert_eq!(
                                        result.options.len(),
                                        explorer_tally_status.results.len()
                                    );
                                    assert_eq!(
                                        result.options.start,
                                        explorer_tally_status.options.start as u8
                                    );
                                    assert_eq!(
                                        result.options.end,
                                        explorer_tally_status.options.end as u8
                                    );
                                } else {
                                    panic!("Wrong tally status. Expected Public")
                                }
                            }
                            Tally::Private { state } => {
                                assert!(explorer_proposal.tally.is_some());
                                if let TallyPrivateStatus(explorer_tally_status) =
                                    explorer_proposal.tally.as_ref().unwrap()
                                {
                                    match state {
                                        PrivateTallyState::Encrypted { encrypted_tally: _ } => {
                                            assert!(
                                                explorer_tally_status.results.is_none(),
                                                "BUG NPG-3369 fixed"
                                            )
                                        }
                                        PrivateTallyState::Decrypted { result } => {
                                            let explorer_tally_result =
                                                explorer_tally_status.results.as_ref().unwrap();
                                            assert_eq!(
                                                result.results.len(),
                                                explorer_tally_result.len()
                                            );
                                            let matching_results = result
                                                .results
                                                .iter()
                                                .zip(explorer_tally_result.iter())
                                                .filter(|&(a, b)| &a.to_string() == b)
                                                .count();
                                            assert_eq!(matching_results, result.results.len());
                                            assert_eq!(
                                                result.options.len(),
                                                explorer_tally_result.len()
                                            );
                                            assert_eq!(
                                                result.options.start,
                                                explorer_tally_status.options.start as u8
                                            );
                                            assert_eq!(
                                                result.options.end,
                                                explorer_tally_status.options.end as u8
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        assert_eq!(
                            vote_proposal_status.votes_cast,
                            explorer_proposal.votes.total_count as usize
                        );
                        if vote_proposal_status.votes_cast == 0 {
                            assert!(explorer_proposal.votes.edges.is_empty());
                        } else {
                            let explorer_votes = &explorer_proposal.votes.edges;
                            assert_eq!(explorer_votes.len(), vote_proposal_status.votes_cast);
                            let votes = proposal_votes
                                .get(&vote_plan_status.id.to_string())
                                .unwrap()
                                .get(&vote_proposal_status.proposal_id.to_string())
                                .unwrap();

                            for vote in votes {
                                for explorer_vote in explorer_votes {
                                    if vote.0.public_key().to_string()
                                        == explorer_vote.node.address.id
                                    {
                                        match &explorer_vote.node.payload {
                                            VotePayloadPublicStatus(choice) => {
                                                assert_eq!(choice.choice as u8, vote.1.as_byte())
                                            }
                                            VotePayloadPrivateStatus(_) => todo!(),
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        assert_eq!(vote_plan_statuses.len(), matching_vote_plans);
    }
}
