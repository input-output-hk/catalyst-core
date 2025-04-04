use super::{build_path_for_challenge, Calculation, OutputFormat};
use crate::types::{challenge::Challenge, proposal::Proposal};
use color_eyre::eyre::Result;
use jormungandr_lib::{
    crypto::hash::Hash,
    interfaces::{VotePlanStatus, VoteProposalStatus},
};
use std::{collections::HashMap, fs::File, io::BufWriter, path::Path};

type CleanedVitSSData = (
    HashMap<Hash, Proposal>,
    HashMap<Hash, VoteProposalStatus>,
    HashMap<i32, Challenge>,
);

pub fn write_results(
    path: &Path,
    format: OutputFormat,
    results: impl IntoIterator<Item = (Challenge, Vec<Calculation>)>,
) -> Result<()> {
    for (challenge, calculations) in results {
        let output_path = build_path_for_challenge(path, &challenge.title);

        match format {
            OutputFormat::Json => write_json(&output_path, &calculations)?,
            OutputFormat::Csv => write_csv(&output_path, &calculations)?,
        };
    }

    Ok(())
}

fn write_json(path: &Path, results: &[Calculation]) -> Result<()> {
    let writer = BufWriter::new(File::options().write(true).open(path)?);
    serde_json::to_writer(writer, &results)?;

    Ok(())
}

fn write_csv(path: &Path, results: &[Calculation]) -> Result<()> {
    let mut writer = csv::Writer::from_path(path)?;
    for record in results {
        writer.serialize(record)?;
    }
    writer.flush()?;

    Ok(())
}

pub fn vecs_to_maps(
    proposals: Vec<Proposal>,
    voteplans: Vec<VotePlanStatus>,
    challenges: Vec<Challenge>,
) -> Result<CleanedVitSSData> {
    let proposals_map: HashMap<_, _> = proposals
        .into_iter()
        .map(|prop| {
            let id = String::from_utf8(prop.chain_proposal_id.clone()).unwrap();
            let hash = Hash::from_hex(&id)?;
            Ok((hash, prop))
        })
        .collect::<Result<_>>()?;

    let voteplan_proposals = voteplans
        .into_iter()
        .flat_map(|plan| plan.proposals)
        .map(|prop| (prop.proposal_id, prop))
        .collect();

    let challenge_map = challenges.into_iter().map(|c| (c.id, c)).collect();

    Ok((proposals_map, voteplan_proposals, challenge_map))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{rewards::proposers::Calculation, utils::json_from_file};
    use assert_fs::TempDir;
    use std::fs::read_to_string;

    #[test]
    fn write_csv_adds_header() {
        let header = [
            "internal_id",
            "proposal_id",
            "proposal",
            "overall_score",
            "yes",
            "no",
            "result",
            "meets_approval_threshold",
            "requested_dollars",
            "status",
            "fund_depletion",
            "not_funded_reason",
            "link_to_ideascale",
        ];
        let calculations = [Calculation::default()];

        let dir = TempDir::new().unwrap();
        let file = dir.join("file.csv");

        File::create(&file).unwrap();
        write_csv(&file, &calculations).unwrap();

        let contents = read_to_string(&file).unwrap();
        let first_line = contents.lines().next().unwrap();
        assert_eq!(first_line, header.join(","));
    }

    #[test]
    fn can_read_data_from_file() {
        let dir = TempDir::new().unwrap();
        let proposals_file = dir.join("proposals");
        let voteplans_file = dir.join("voteplans");
        let challenges_file = dir.join("challenges");

        let empty = "[]";
        std::fs::write(&proposals_file, empty).unwrap();
        std::fs::write(&voteplans_file, empty).unwrap();
        std::fs::write(&challenges_file, empty).unwrap();

        let proposals: Vec<Proposal> = json_from_file(proposals_file).unwrap();
        let voteplans: Vec<VotePlanStatus> = json_from_file(voteplans_file).unwrap();
        let challenges: Vec<Challenge> = json_from_file(challenges_file).unwrap();

        assert_eq!(proposals, vec![]);
        assert_eq!(voteplans, vec![]);
        assert_eq!(challenges, vec![]);
    }
}
