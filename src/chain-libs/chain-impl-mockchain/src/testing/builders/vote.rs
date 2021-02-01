use crate::{
    certificate::{PrivateTallyDecrypted, PrivateTallyDecryptedProposal},
    testing::data::CommitteeMembersManager,
    vote::VotePlanStatus,
};

pub fn decrypt_tally(
    vote_plan_status: &VotePlanStatus,
    members: &CommitteeMembersManager,
) -> PrivateTallyDecrypted {
    let encrypted_tally = vote_plan_status
        .proposals
        .iter()
        .map(|proposal| {
            let tally_state = proposal.tally.as_ref().unwrap();
            let encrypted_tally = tally_state.private_encrypted().unwrap().0.clone();
            let max_votes = tally_state.private_total_power().unwrap();
            (encrypted_tally, max_votes)
        })
        .collect::<Vec<_>>();

    let absolute_max_votes = encrypted_tally
        .iter()
        .map(|(_encrypted_tally, max_votes)| *max_votes)
        .max()
        .unwrap();
    let table = chain_vote::TallyOptimizationTable::generate_with_balance(absolute_max_votes, 1);

    let proposals = encrypted_tally
        .into_iter()
        .map(|(encrypted_tally, max_votes)| {
            let decrypt_shares = members
                .members()
                .iter()
                .map(|member| member.secret_key())
                .map(|secret_key| encrypted_tally.finish(secret_key).1)
                .collect::<Vec<_>>();
            let tally_state = encrypted_tally.state();
            let tally =
                chain_vote::tally(max_votes, &tally_state, &decrypt_shares, &table).unwrap();
            PrivateTallyDecryptedProposal {
                decrypt_shares: decrypt_shares.into_boxed_slice(),
                tally_result: tally.votes.into_boxed_slice(),
            }
        })
        .collect::<Vec<_>>();

    PrivateTallyDecrypted::new(proposals)
}
