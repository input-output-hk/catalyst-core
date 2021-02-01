use crate::{
    certificate::{PrivateTallyDecrypted, PrivateTallyDecryptedProposal}, testing::data::CommitteeMembersManager, vote::VotePlanStatus,
};
pub fn build_tally_decrypt_share(
    vote_plan_status: &VotePlanStatus,
    members: &CommitteeMembersManager,
) -> PrivateTallyDecrypted {
    let shares = vote_plan_status
        .proposals
        .iter()
        .map(|proposal| {
            let tally_state = proposal
                .tally
                .as_ref()
                .unwrap();
            let encrypted_tally = tally_state.private_encrypted()
                .unwrap()
                .0
                .clone();
            let max_votes = tally_state.private_total_power().unwrap();
            (encrypted_tally, max_votes)
        })
        .map(|(encrypted_tally, max_votes)| {
            let decrypt_shares = members
                .members()
                .iter()
                .map(|member| member.secret_key())
                .map(|secret_key| encrypted_tally.finish(secret_key).1)
                .collect::<Vec<_>>();
            let tally_state= encrypted_tally.state();
            let table = chain_vote::TallyOptimizationTable::generate_with_balance(max_votes, 1);
            let tally = chain_vote::tally(max_votes, &tally_state, &decrypt_shares, &table).unwrap();
            PrivateTallyDecryptedProposal {
                shares: decrypt_shares.into_boxed_slice(), 
                decrypted: tally.votes.into_boxed_slice(),
            }
        })
        .collect::<Vec<_>>();

    PrivateTallyDecrypted::new(shares)
}
