use super::{AlignmentScore, AuditabilityScore, FeasibilityScore};
use std::{fs::File, path::Path, str::FromStr};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("Invalid proposal data: {0}")]
    InvalidProposalData(String),
}

pub fn store_score_into_proposal(
    proposal: &mut serde_json::Value,
    alignment_score: AlignmentScore,
    feasibility_score: FeasibilityScore,
    auditability_score: AuditabilityScore,
) -> Result<(), Error> {
    let files_url_data = proposal
        .get_mut("files_url")
        .ok_or_else(|| Error::InvalidProposalData("missing field \"files_url\"".to_string()))?;

    let mut files_url_object = serde_json::Value::from_str(
        files_url_data
            .as_str()
            .ok_or_else(|| {
                Error::InvalidProposalData("data inside \"files_url\" not a string".to_string())
            })?
            .replace('\'', "\"")
            .as_str(),
    )?;

    let values = files_url_object.as_object_mut().ok_or_else(|| {
        Error::InvalidProposalData("data inside \"files_url\" not json encoded".to_string())
    })?;
    values.insert("alignment_score".to_string(), alignment_score.0.into());
    values.insert("feasibility_score".to_string(), feasibility_score.0.into());
    values.insert(
        "auditability_score".to_string(),
        auditability_score.0.into(),
    );

    *files_url_data = files_url_object.to_string().replace('"', "'").into();

    Ok(())
}

pub fn store_proposals_into_file(
    path: &Path,
    proposals: Vec<serde_json::Value>,
) -> Result<(), Error> {
    let mut file = File::create(path)?;
    serde_json::to_writer_pretty(&mut file, &proposals)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_into_proposal() {
        let mut proposal = serde_json::json!(
            {
                "files_url": serde_json::json!(
                    {
                        "some_data": "data"
                    }
                ).to_string()
            }
        );

        store_score_into_proposal(
            &mut proposal,
            AlignmentScore(0.8),
            FeasibilityScore(0.9),
            AuditabilityScore(2.5),
        )
        .unwrap();
        assert_eq!(
            proposal,
            serde_json::json!(
                {
                    "files_url": serde_json::json!(
                        {
                            "some_data": "data",
                            "alignment_score": 0.8,
                            "feasibility_score": 0.9,
                            "auditability_score": 2.5
                        }
                    ).to_string().replace('"', "'")
                }
            )
        )
    }
}
