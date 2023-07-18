use super::{AligmentScore, AuditabilityScore, FeasibilityScore, ProposalId};
use rusqlite::Connection;
use std::{path::PathBuf, str::FromStr};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub fn store_scores_in_sqllite_db(
    db: &PathBuf,
    proposal_id: ProposalId,
    aligment_score: AligmentScore,
    feasibility_score: FeasibilityScore,
    auditability_score: AuditabilityScore,
) -> Result<(), Error> {
    let conn = Connection::open(db)?;

    let mut statement =
        conn.prepare("SELECT proposal_files_url FROM proposals WHERE proposal_id = ?1;")?;
    let proposal_files_url: String = statement.query_row([proposal_id.0], |row| row.get(0))?;

    let mut current;
    if proposal_files_url.is_empty() {
        current = serde_json::json!(
            {
                "aligment_score": aligment_score.0,
                "feasibility_score": feasibility_score.0,
                "auditability_score": auditability_score.0
            }
        );
    } else {
        current = serde_json::Value::from_str(&proposal_files_url)?;
        let values = current.as_object_mut().unwrap();
        values.insert("aligment_score".to_string(), aligment_score.0.into());
        values.insert("feasibility_score".to_string(), feasibility_score.0.into());
        values.insert(
            "auditability_score".to_string(),
            auditability_score.0.into(),
        );
    }

    conn.execute(
        "UPDATE proposals SET proposal_files_url = ?1 WHERE proposal_id = ?2;",
        (current.to_string(), &proposal_id.0),
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_scores_in_sqllite_db() {
        let db = PathBuf::from("src/proposal_score/test_data/fund9.sqlite3");
        let proposal_id = ProposalId(423260);
        let aligment_score = AligmentScore(0.8);
        let feasibility_score = FeasibilityScore(0.9);
        let auditability_score = AuditabilityScore(2.5);

        store_scores_in_sqllite_db(
            &db,
            proposal_id,
            aligment_score,
            feasibility_score,
            auditability_score,
        )
        .unwrap();
    }
}
