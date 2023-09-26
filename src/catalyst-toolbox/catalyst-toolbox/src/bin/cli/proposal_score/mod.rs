use clap::Parser;
use color_eyre::Report;
use std::path::PathBuf;

/// This command takes a csv file with the reviews as an input,
/// calculates scores for each proposal based on reviews and `allocated_weight` and `not_allocated_weight` values,
/// then stores result into sqlite3 database into proposals table, proposal_files_url column in the json format.
/// Samples for csv file and sqlite3 database format you can find in `src/proposal_score/test_data` folder.
#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub struct ProposalScore {
    /// Allocated review weight value
    #[clap(long)]
    allocated_weight: f64,

    /// Not allocated review weight value
    #[clap(long)]
    not_allocated_weight: f64,

    /// Path to the input csv file with the reviews
    #[clap(long)]
    reviews_path: PathBuf,

    /// Path to the input json file with the proposals file, in which resulted output will rewrite its content
    #[clap(long)]
    proposals_path: PathBuf,
}

impl ProposalScore {
    pub fn exec(self) -> Result<(), Report> {
        let reviews =
            catalyst_toolbox::proposal_score::load::load_reviews_from_csv(&self.reviews_path)?;
        let mut proposals =
            catalyst_toolbox::proposal_score::load::load_proposals_from_json(&self.proposals_path)?;

        for (proposal_id, (alignment_reviews, feasibility_reviews, auditability_reviews)) in reviews
        {
            let (alignment_score, feasibility_score, auditability_score) =
                catalyst_toolbox::proposal_score::calc_score(
                    self.allocated_weight,
                    self.not_allocated_weight,
                    &alignment_reviews,
                    &feasibility_reviews,
                    &auditability_reviews,
                )?;

            let proposal = proposals.get_mut(&proposal_id).ok_or_else(|| {
                color_eyre::eyre::eyre!("Proposal with id {} not found", proposal_id.0)
            })?;
            catalyst_toolbox::proposal_score::store::store_score_into_proposal(
                proposal,
                alignment_score,
                feasibility_score,
                auditability_score,
            )?;
        }

        catalyst_toolbox::proposal_score::store::store_proposals_into_file(
            &self.proposals_path,
            proposals.into_values().collect(),
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proposal_score_test() {
        let proposal_score = ProposalScore {
            allocated_weight: 0.8,
            not_allocated_weight: 0.2,
            reviews_path: PathBuf::from("src/proposal_score/test_data/reviews-example.csv"),
            proposals_path: PathBuf::from("src/proposal_score/test_data/proposals.json"),
        };

        proposal_score.exec().unwrap();
    }
}
