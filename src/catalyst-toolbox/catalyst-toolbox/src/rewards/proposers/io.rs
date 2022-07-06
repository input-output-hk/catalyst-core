use crate::http::HttpClient;
use color_eyre::eyre::Result;
use jormungandr_lib::{
    crypto::hash::Hash,
    interfaces::{VotePlanStatus, VoteProposalStatus},
};
use log::{info, warn};
use std::{collections::HashMap, fs::File, io::BufWriter, path::Path};
use vit_servicing_station_lib::db::models::{challenges::Challenge, proposals::Proposal};

use serde::Deserialize;

use super::Calculation;

type VitSSData = (Vec<Proposal>, Vec<VotePlanStatus>, Vec<Challenge>);
type CleanedVitSSData = (
    HashMap<Hash, Proposal>,
    HashMap<Hash, VoteProposalStatus>,
    HashMap<i32, Challenge>,
);

pub fn write_json(path: &Path, results: &[Calculation]) -> Result<()> {
    let writer = BufWriter::new(File::options().write(true).open(path)?);
    serde_json::to_writer(writer, &results)?;

    Ok(())
}

pub fn write_csv(path: &Path, results: &[Calculation]) -> Result<()> {
    let mut writer = csv::Writer::from_path(path)?;
    for record in results {
        writer.serialize(record)?;
    }
    writer.flush()?;

    Ok(())
}

pub fn load_data(
    http: &impl HttpClient,
    vit_station_url: &str,
    proposals: Option<&Path>,
    active_voteplans: Option<&Path>,
    challenges: Option<&Path>,
) -> Result<CleanedVitSSData> {
    let data = match (proposals, active_voteplans, challenges) {
        (Some(p), Some(vp), Some(c)) => {
            info!("loading data from files");
            get_data_from_files(p, vp, c)?
        }
        (None, None, None) => {
            info!("loading data from network: {vit_station_url}");
            get_data_from_network(http, vit_station_url)?
        }
        _else => {
            warn!("warning: not all of --proposals, --active-voteplans and --challenges were set, falling back to network");
            info!("loading data from network: {vit_station_url}");
            get_data_from_network(http, vit_station_url)?
        }
    };

    let (proposals, voteplans, challenges) = data;

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

fn get_data_from_network(http: &impl HttpClient, vit_station_url: &str) -> Result<VitSSData> {
    let proposals = json_from_network(http, format!("{vit_station_url}/api/v0/proposals"))?;
    let voteplans = json_from_network(http, format!("{vit_station_url}/api/v0/vote/active/plans"))?;
    let challenges = json_from_network(http, format!("{vit_station_url}/api/v0/challenges"))?;

    Ok((proposals, voteplans, challenges))
}

pub fn json_from_file<T: for<'a> Deserialize<'a>>(path: impl AsRef<Path>) -> Result<T> {
    Ok(serde_json::from_reader(File::open(path)?)?)
}

pub fn json_from_network<T: for<'a> Deserialize<'a>>(
    http: &impl HttpClient,
    url: impl AsRef<str>,
) -> Result<T> {
    http.get(url.as_ref())?.json()
}

fn get_data_from_files(
    proposals: &Path,
    active_voteplans: &Path,
    challenges: &Path,
) -> Result<VitSSData> {
    let proposals = json_from_file(proposals)?;
    let voteplans = json_from_file(active_voteplans)?;
    let challenges = json_from_file(challenges)?;

    Ok((proposals, voteplans, challenges))
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use super::*;
    use assert_fs::TempDir;
    use reqwest::StatusCode;

    use crate::{
        http::mock::{Method, MockClient, Spec},
        rewards::proposers::Calculation,
    };

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

        let (proposals, voteplans, challenges) =
            get_data_from_files(&proposals_file, &voteplans_file, &challenges_file).unwrap();

        assert_eq!(proposals, vec![]);
        assert_eq!(voteplans, vec![]);
        assert_eq!(challenges, vec![]);
    }

    #[test]
    fn can_read_data_from_network() {
        let mock_client = MockClient::new(|spec| match spec {
            Spec {
                method: Method::Get,
                path: "/api/v0/proposals",
            } => ("[]".to_string(), StatusCode::OK),
            Spec {
                method: Method::Get,
                path: "/api/v0/vote/active/plans",
            } => ("[]".to_string(), StatusCode::OK),
            Spec {
                method: Method::Get,
                path: "/api/v0/challenges",
            } => ("[]".to_string(), StatusCode::OK),
            _ => ("not found".to_string(), StatusCode::NOT_FOUND),
        });

        let (proposals, voteplans, challenges) = get_data_from_network(&mock_client, "").unwrap();

        assert_eq!(proposals, vec![]);
        assert_eq!(voteplans, vec![]);
        assert_eq!(challenges, vec![]);
    }
}
