use super::{AligmentScore, AuditabilityScore, FeasibilityScore, ProposalId};
use rusqlite::Connection;
use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
}

pub fn store_scores_in_sqllite_db(
    db: &PathBuf,
    proposal_id: ProposalId,
    aligment_score: AligmentScore,
    feasibility_score: FeasibilityScore,
    auditability_score: AuditabilityScore,
) -> Result<(), Error> {
    let conn = Connection::open(db)?;

    let json = serde_json::json!(
        {
            "aligment_score": aligment_score.0,
            "feasibility_score": feasibility_score.0,
            "auditability_score": auditability_score.0
        }
    );

    conn.execute(
        "UPDATE proposals SET proposal_files_url = ?1 WHERE proposal_id = ?2;",
        (json.to_string(), &proposal_id.0),
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
        let aligment_score = AligmentScore(0.5);
        let feasibility_score = FeasibilityScore(0.5);
        let auditability_score = AuditabilityScore(0.5);

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
