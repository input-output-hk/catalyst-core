use catalyst_toolbox::{
    community_advisors::models::{ReviewRanking, VeteranRankingRow},
    ideascale::{
        build_challenges, build_fund, build_proposals, fetch_all, CustomFieldTags, Scores, Sponsors,
    },
    utils::csv::{dump_data_to_csv, load_data_from_csv},
};
use color_eyre::Report;
use itertools::Itertools;
use jcli_lib::utils::io as io_utils;
use jormungandr_lib::interfaces::VotePrivacy;
use std::{collections::HashSet, ffi::OsStr};

use structopt::StructOpt;

use serde::de::DeserializeOwned;
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, StructOpt)]
pub enum Ideascale {
    Import(Import),
    Filter(Filter),
}

// We need this type because structopt uses Vec<String> as a special type, so it is not compatible
// with custom parsers feature.
type Filters = Vec<String>;

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab")]
pub struct Import {
    /// Fund number id
    #[structopt(long)]
    fund: usize,

    /// Fund goal explanation
    #[structopt(long)]
    fund_goal: String,

    /// Stage label: stage identifiers that links to assessments scores in ideascale
    #[structopt(long, default_value = "Assess")]
    stage_label: String,

    /// ideascale API token
    #[structopt(long, env = "IDEASCALE_API_TOKEN")]
    api_token: String,

    /// Fund approval threshold setting
    #[structopt(long)]
    threshold: i64,

    /// either "public" or "private"
    #[structopt(long)]
    chain_vote_type: VotePrivacy,

    /// Path to folder where fund, challenges and proposals json files will be dumped
    #[structopt(long)]
    output_dir: PathBuf,

    /// Path to proposal scores csv file
    #[structopt(long)]
    scores: Option<PathBuf>,

    /// Path to proposal scores csv file
    #[structopt(long)]
    sponsors: Option<PathBuf>,

    /// Path to json or json like file containing tag configuration for ideascale custom fields
    #[structopt(long)]
    tags: Option<PathBuf>,

    /// Path to json or json like file containing list of excluded proposal ids
    #[structopt(long)]
    excluded_proposals: Option<PathBuf>,

    /// Ideascale stages list,
    #[structopt(long, parse(from_str=parse_from_csv), default_value = "Governance phase;Assess QA")]
    stages_filters: Filters,
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab")]
pub struct Filter {
    #[structopt(long)]
    input: PathBuf,

    #[structopt(long)]
    output: Option<PathBuf>,
}

impl Ideascale {
    pub fn exec(&self) -> Result<(), Report> {
        match self {
            Ideascale::Import(import) => import.exec(),
            Ideascale::Filter(filter) => filter.exec(),
        }
    }
}

impl Import {
    fn exec(&self) -> Result<(), Report> {
        let Import {
            fund,
            fund_goal,
            stage_label,
            api_token,
            threshold,
            chain_vote_type,
            output_dir: save_folder,
            scores,
            sponsors,
            tags,
            excluded_proposals,
            stages_filters,
        } = self;

        let tags: CustomFieldTags = if let Some(tags_path) = tags {
            read_json_from_file(tags_path)?
        } else {
            Default::default()
        };

        let excluded_proposals: HashSet<u32> = if let Some(excluded_path) = excluded_proposals {
            read_json_from_file(excluded_path)?
        } else {
            Default::default()
        };

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_io()
            .enable_time()
            .build()?;

        let scores = read_scores_file(scores)?;
        let sponsors = read_sponsors_file(sponsors)?;
        let idescale_data = runtime.block_on(fetch_all(
            *fund,
            &stage_label.to_lowercase(),
            &stages_filters.iter().map(AsRef::as_ref).collect::<Vec<_>>(),
            &excluded_proposals,
            api_token.clone(),
        ))?;

        let funds = build_fund(*fund as i32, fund_goal.clone(), *threshold);
        let challenges = build_challenges(*fund as i32, &idescale_data, sponsors);
        let mut proposals = build_proposals(
            &idescale_data,
            &challenges,
            &scores,
            &chain_vote_type.to_string(),
            *fund,
            &tags,
        );

        let mut challenges: Vec<_> = challenges.values().collect();
        // even if final id type is string, they are just sequentially added, so it should be safe
        // to parse and unwrap here
        challenges.sort_by_key(|c| c.id.parse::<i32>().unwrap());
        proposals.sort_by_key(|p| p.proposal_id.clone());

        dump_content_to_file(
            funds,
            save_folder
                .join(format!("fund{}_funds.json", fund))
                .as_path(),
        )?;

        dump_content_to_file(
            challenges,
            save_folder
                .join(format!("fund{}_challenges.json", fund))
                .as_path(),
        )?;

        dump_content_to_file(
            proposals,
            save_folder
                .join(format!("fund{}_proposals.json", fund))
                .as_path(),
        )?;

        Ok(())
    }
}

impl Filter {
    fn output_file(input: &Path, output: Option<&Path>) -> PathBuf {
        if let Some(output) = output {
            output.to_path_buf()
        } else {
            let name = input.file_name().and_then(OsStr::to_str).unwrap_or("");
            let name = format!("{name}.output");
            let temp = input.with_file_name(name);
            println!("no output specified, writing to {}", temp.to_string_lossy());
            temp
        }
    }

    fn filter_rows(rows: &[VeteranRankingRow]) -> Vec<VeteranRankingRow> {
        let groups = rows
            .iter()
            .group_by(|row| (&row.assessor, &row.proposal_id));
        groups
            .into_iter()
            .flat_map(|(_, group)| {
                let group = group.collect_vec();
                let excellent = group
                    .iter()
                    .filter(|row| row.score() == ReviewRanking::Excellent)
                    .count();
                let good = group
                    .iter()
                    .filter(|row| row.score() == ReviewRanking::Good)
                    .count();
                let filtered = group
                    .iter()
                    .filter(|row| row.score() == ReviewRanking::FilteredOut)
                    .count();

                use std::cmp::max;
                let max_count = max(excellent, max(good, filtered));

                let include_excellent = excellent == max_count;
                let include_good = good == max_count;
                let include_filtered = filtered == max_count;

                group.into_iter().filter(move |row| match row.score() {
                    ReviewRanking::Excellent => include_excellent,
                    ReviewRanking::Good => include_good,
                    ReviewRanking::FilteredOut => include_filtered,
                    ReviewRanking::NA => true, // if unknown, ignore
                })
            })
            .cloned()
            .collect()
    }

    fn exec(&self) -> Result<(), Report> {
        let Self { input, output } = self;
        let output = Self::output_file(input, output.as_deref());

        let rows = load_data_from_csv::<_, b','>(input)?;
        let rows = Self::filter_rows(&rows);

        dump_data_to_csv(&rows, &output)?;

        Ok(())
    }
}

fn dump_content_to_file(content: impl Serialize, file_path: &Path) -> Result<(), Report> {
    let writer = jcli_lib::utils::io::open_file_write(&Some(file_path))?;
    serde_json::to_writer_pretty(writer, &content)?;
    Ok(())
}

fn read_json_from_file<T: DeserializeOwned>(file_path: &Path) -> Result<T, Report> {
    let reader = io_utils::open_file_read(&Some(file_path))?;
    Ok(serde_json::from_reader(reader)?)
}

fn parse_from_csv(s: &str) -> Filters {
    s.split(';').map(|x| x.to_string()).collect()
}

fn read_scores_file(path: &Option<PathBuf>) -> Result<Scores, Report> {
    let mut scores = Scores::new();
    if let Some(path) = path {
        let mut reader = csv::Reader::from_path(path)?;
        for record in reader.records() {
            let record = record?;
            let proposal_id: u32 = record
                .get(0)
                .expect("Proposal ids should be present in scores file second column")
                .parse()
                .expect("Proposal ids should be integers");
            let rating_given: f32 = record
                .get(1)
                .expect("Ratings should be present in scores file third column")
                .parse()
                .expect("Ratings should be floats [0, 5]");
            scores.insert(proposal_id, rating_given);
        }
    }
    Ok(scores)
}

fn read_sponsors_file(path: &Option<PathBuf>) -> Result<Sponsors, Report> {
    let mut sponsors = Sponsors::new();

    if let Some(path) = path {
        let mut reader = csv::Reader::from_path(path)?;
        for record in reader.records() {
            let record = record?;
            let challenge_url: String = record
                .get(0)
                .expect("Challenge url should be present in scores file first column")
                .to_string();
            let sponsor_name: String = record
                .get(1)
                .expect("Sponsor name should be present in scores file second column")
                .to_string();
            sponsors.insert(challenge_url, sponsor_name);
        }
    }
    Ok(sponsors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correctly_formats_output_file_for_filter() {
        let input = PathBuf::from("/foo/bar/file.txt");
        let output = PathBuf::from("/baz/qux/output.txt");

        let result = Filter::output_file(&input, Some(&output));
        assert_eq!(result, output);

        let result = Filter::output_file(&input, None);
        assert_eq!(result, PathBuf::from("/foo/bar/file.txt.output"));
    }

    #[test]
    fn filters_rows_correctly() {
        use ReviewRanking::*;

        let pid = String::from("pid");
        let assessor = String::from("assessor");
        let first = VeteranRankingRow::new(pid.clone(), assessor.clone(), "1".into(), Excellent);
        let second = VeteranRankingRow::new(pid.clone(), assessor.clone(), "2".into(), Excellent);
        let third = VeteranRankingRow::new(pid.clone(), assessor.clone(), "3".into(), Good);

        let rows = vec![first.clone(), second.clone(), third];
        let expected_rows = vec![first, second];

        assert_eq!(Filter::filter_rows(&rows), expected_rows);
    }
}
