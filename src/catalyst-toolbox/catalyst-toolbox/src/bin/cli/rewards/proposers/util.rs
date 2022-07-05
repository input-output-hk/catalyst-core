use catalyst_toolbox::http::HttpClient;
use color_eyre::eyre::Result;
use jormungandr_lib::{
    crypto::hash::Hash,
    interfaces::{VotePlanStatus, VoteProposalStatus},
};
use log::{info, warn};
use std::{collections::HashMap, fs::File, path::Path};
use vit_servicing_station_lib::db::models::{challenges::Challenge, proposals::Proposal};

use serde::Deserialize;

type VitSSData = (Vec<Proposal>, Vec<VotePlanStatus>, Vec<Challenge>);
type CleanedVitSSData = (
    HashMap<Hash, Proposal>,
    HashMap<Hash, VoteProposalStatus>,
    HashMap<i32, Challenge>,
);

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
