use std::collections::{HashMap, HashSet};

use crate::diesel::BoolExpressionMethods;
use bigdecimal::{BigDecimal, FromPrimitive};
use color_eyre::{
    eyre::{bail, ensure, eyre},
    Result,
};
use diesel::{JoinOnDsl, QueryDsl, RunQueryDsl};
use once_cell::sync::Lazy;
use serde_json::{from_value, Value};

use crate::{
    db::{schema, Db},
    model::Rego,
    model::SlotNo,
};

static METADATA_KEY: Lazy<BigDecimal> = Lazy::new(|| BigDecimal::from_isize(61284).unwrap());
static SIGNATURE_KEY: Lazy<BigDecimal> = Lazy::new(|| BigDecimal::from_isize(61285).unwrap());

type Row = (i64, Option<Value>, BigDecimal);

impl Db {
    pub fn vote_registrations(&self, slot_no: Option<SlotNo>) -> Result<Vec<Rego>> {
        use diesel::ExpressionMethods;
        use schema::*;

        let slot_no = match slot_no.map(|s| s.0.try_into()) {
            None => i64::MAX,
            Some(Ok(i)) => i,
            Some(Err(e)) => bail!(e),
        };

        let result: Vec<Row> = self.exec(move |conn| {
            let slot_no_predicate = block::slot_no.le(slot_no);
            let metadata_key_predicate = tx_metadata::key
                .eq(&*METADATA_KEY)
                .or(tx_metadata::key.eq(&*SIGNATURE_KEY));

            let select = (tx_metadata::tx_id, tx_metadata::json, tx_metadata::key);

            tx::table
                .inner_join(tx_metadata::table.on(tx::id.eq(tx_metadata::tx_id)))
                .inner_join(block::table.on(block::id.eq(tx::block_id)))
                .filter(slot_no_predicate)
                .filter(metadata_key_predicate)
                .select(select)
                .load(conn)
        })?;

        process(result)
    }
}

/// The database query returns
fn process(rows: Vec<Row>) -> Result<Vec<Rego>> {
    let (metadatas, signatures): (Vec<_>, _) =
        rows.into_iter().partition(|row| row.2 == *METADATA_KEY);

    assert!(signatures.iter().all(|r| r.2 == *SIGNATURE_KEY));

    let metadatas = make_hashmap(metadatas);
    let mut signatures = make_hashmap(signatures);

    ensure!(
        metadatas.keys().collect::<HashSet<_>>() == signatures.keys().collect::<HashSet<_>>(),
        "metadatas and signatures had different keys"
    );

    let mut regos = Vec::with_capacity(metadatas.keys().count());

    for (tx_id, metadata) in metadatas {
        // This unwrap is fine because we asserted above that the key sets are equal
        // If we want to parallelize, consider swapping this out for `dashmap`, which is threadsafe
        let signature = signatures.remove(&tx_id).unwrap();
        if let Ok(rego) = build_rego(tx_id, metadata, signature) {
            regos.push(rego);
        }
    }

    Ok(regos)
}

/// Attempt to build a registration. Extracted into a function to allow for `?` operator while
/// not short-circuiting
fn build_rego(tx_id: i64, metadata: Option<Value>, signature: Option<Value>) -> Result<Rego> {
    let tx_id = u64::try_from(tx_id)?.into();
    let metadata = metadata.ok_or(eyre!("metadata is missing"))?;
    let signature = signature.ok_or(eyre!("signature is missing"))?;

    let rego = Rego {
        tx_id,
        metadata: from_value(metadata)?,
        signature: from_value(signature)?,
    };

    Ok(rego)
}

/// Collect into a hashmap by id, discarding the key
fn make_hashmap(v: Vec<Row>) -> HashMap<i64, Option<Value>> {
    v.into_iter().map(|(id, json, _key)| (id, json)).collect()
}

#[cfg(test)]
mod tests {
    use crate::model::{Delegations, RegoMetadata, RegoSignature};
    use serde_json::json;

    use super::*;

    fn dummy_metadata() -> Value {
        json!({
            "1": "legacy",
            "2": "stakevkey",
            "3": "rewardsaddr",
            "4": 4,
        })
    }

    fn dummy_signature() -> Value {
        json!({ "1": "signature" })
    }

    #[test]
    fn process_rows_happy_path() {
        let rows = vec![
            (1, Some(dummy_metadata()), METADATA_KEY.clone()),
            (1, Some(dummy_signature()), SIGNATURE_KEY.clone()),
        ];

        let regos = process(rows).unwrap();

        assert_eq!(
            regos,
            vec![Rego {
                tx_id: 1.into(),
                signature: RegoSignature {
                    signature: "signature".into(),
                },
                metadata: RegoMetadata {
                    delegations: Delegations::Legacy("legacy".into()),
                    stake_vkey: "stakevkey".into(),
                    rewards_addr: "rewardsaddr".into(),
                    slot: 4.into(),
                    purpose: 0.into(),
                }
            }]
        )
    }

    #[test]
    fn ignores_bad_or_missing_json() {
        let rows = vec![
            (1, Some(json!("bad json")), METADATA_KEY.clone()),
            (1, None, SIGNATURE_KEY.clone()),
        ];

        let regos = process(rows).unwrap();

        assert!(regos.is_empty());
    }
}
