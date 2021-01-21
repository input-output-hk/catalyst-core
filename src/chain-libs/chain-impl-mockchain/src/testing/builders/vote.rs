use crate::{
    certificate::TallyDecryptShares, testing::data::CommitteeMembersManager, vote::VotePlanStatus,
};
pub fn build_tally_decrypt_share(
    vote_plan_status: &VotePlanStatus,
    members: &CommitteeMembersManager,
) -> TallyDecryptShares {
    let shares = vote_plan_status
        .proposals
        .iter()
        .map(|proposal| {
            proposal
                .tally
                .as_ref()
                .unwrap()
                .private_encrypted()
                .unwrap()
                .0
                .clone()
        })
        .map(|encrypted_tally| {
            members
                .members()
                .iter()
                .map(|member| member.secret_key())
                .map(|secret_key| encrypted_tally.finish(secret_key).1)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    TallyDecryptShares::new(shares)
}
