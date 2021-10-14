use serde::Deserialize;

pub type VeteranAdvisorId = String;
pub type VeteranAdvisorReward = rust_decimal::Decimal;

pub type VeteranAdvisorRewards = Vec<(VeteranAdvisorId, VeteranAdvisorReward)>;

#[derive(Deserialize)]
pub struct VeteranReviews {
    name: VeteranAdvisorId,
    #[serde(alias = "No. of Reviews")]
    number_of_reviews: usize,
}

pub fn calculate_veteran_advisors_rewards(
    veteran_reviews: &[VeteranReviews],
    base_rewards: VeteranAdvisorReward,
) -> VeteranAdvisorRewards {
    let total_reviews: VeteranAdvisorReward = VeteranAdvisorReward::from(
        veteran_reviews
            .iter()
            .map(|vr| vr.number_of_reviews)
            .sum::<usize>(),
    );

    veteran_reviews
        .iter()
        .map(|vr| {
            (
                vr.name.clone(),
                (VeteranAdvisorReward::from(vr.number_of_reviews) / total_reviews) * base_rewards,
            )
        })
        .collect()
}
