use std::{
    borrow::Cow,
    collections::HashSet,
    fs::File,
    io::BufWriter,
    path::{Path, PathBuf},
};

use catalyst_toolbox::{
    http::HttpClient,
    rewards::proposers::{
        proposer_rewards, Calculation, OutputFormat, ProposerRewards, ProposerRewardsInputs,
    },
};
use color_eyre::eyre::Result;
use once_cell::sync::Lazy;
use regex::Regex;

use self::util::load_data;

mod util;

pub fn rewards(
    ProposerRewards {
        output,
        block0,
        proposals,
        excluded_proposals,
        active_voteplans,
        challenges,
        committee_keys,
        total_stake_threshold,
        approval_threshold,
        output_format,
        vit_station_url,
    }: &ProposerRewards,
    http: &impl HttpClient,
) -> Result<()> {
    let (proposals, voteplans, challenges) = load_data(
        http,
        vit_station_url,
        proposals.as_deref(),
        active_voteplans.as_deref(),
        challenges.as_deref(),
    )?;

    let block0_config = serde_yaml::from_reader(File::open(block0)?)?;

    let excluded_proposals = match excluded_proposals {
        Some(path) => serde_json::from_reader(File::open(path)?)?,
        None => HashSet::new(),
    };
    let committee_keys = match committee_keys {
        Some(path) => serde_json::from_reader(File::open(path)?)?,
        None => vec![],
    };

    let results = proposer_rewards(ProposerRewardsInputs {
        block0_config,
        proposals,
        voteplans,
        challenges,
        excluded_proposals,
        committee_keys,
        total_stake_threshold: *total_stake_threshold,
        approval_threshold: *approval_threshold,
    })?;

    for (challenge, calculations) in results {
        let output_path = build_path_for_challenge(output, &challenge.title);

        match output_format {
            OutputFormat::Json => write_json(&output_path, &calculations)?,
            OutputFormat::Csv => write_csv(&output_path, &calculations)?,
        };
    }

    Ok(())
}

fn build_path_for_challenge(path: &Path, challenge_name: &str) -> PathBuf {
    let challenge_name = sanitize_name(challenge_name);
    let ext = path.extension();

    let mut path = path.with_extension("").as_os_str().to_owned();
    path.push("_");
    path.push(&*challenge_name);
    let path = PathBuf::from(path);

    match ext {
        Some(ext) => path.with_extension(ext),
        None => path,
    }
}

fn sanitize_name(name: &str) -> Cow<'_, str> {
    static REMOVE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"[^-\w.]"#).unwrap());
    static REPLACE_UNDERSCORE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#" |:"#).unwrap()); // space or colon
                                                                                                //
    let name = REPLACE_UNDERSCORE_REGEX.replace_all(name, "_");
    match name {
        Cow::Borrowed(borrow) => REMOVE_REGEX.replace_all(borrow, ""),
        Cow::Owned(owned) => {
            let result = REMOVE_REGEX.replace_all(&owned, "");
            Cow::Owned(result.to_string())
        }
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_replaces_correctly() {
        assert_eq!(sanitize_name("asdf"), "asdf");
        // colons and spaces replaced with underscores
        assert_eq!(sanitize_name("a b:c"), "a_b_c");
        // other symbols removed
        assert_eq!(sanitize_name("a£$%^&*()bc"), "abc");
        // . and - are allowed
        assert_eq!(sanitize_name("a.b-c"), "a.b-c");
        // all together
        assert_eq!(sanitize_name("foo$%. bar:baz"), "foo._bar_baz");
    }

    #[test]
    fn test_build_path() {
        let path = "/some/path.ext";
        let challenge = "challenge";
        let built_path = build_path_for_challenge(Path::new(path), challenge);
        assert_eq!(built_path, PathBuf::from("/some/path_challenge.ext"));
    }

    #[test]
    fn test_build_path_hidden_file() {
        let path = "/some/.path.ext";
        let challenge = "challenge";
        let built_path = build_path_for_challenge(Path::new(path), challenge);
        assert_eq!(built_path, PathBuf::from("/some/.path_challenge.ext"));
    }
}
