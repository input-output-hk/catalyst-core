use std::fs::File;
use std::io::{BufWriter, Write};

use crate::{db::inner::DbQuery, Db};

use crate::data::{SignedRegistration, SlotNo};
use crate::db::schema::{block, tx, tx_metadata};
use bigdecimal::{BigDecimal, FromPrimitive};
use color_eyre::eyre::{eyre, Result};
use color_eyre::{Help, Report};
use diesel::RunQueryDsl;
use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, PgAnyJsonExpressionMethods,
    PgJsonbExpressionMethods, QueryDsl,
};
use itertools::Itertools;
use once_cell::sync::Lazy;
use serde_json::Value;

use tracing::{debug, info};

static METADATA_KEY: Lazy<BigDecimal> = Lazy::new(|| BigDecimal::from_isize(61284).unwrap());
static SIGNATURE_KEY: Lazy<BigDecimal> = Lazy::new(|| BigDecimal::from_isize(61285).unwrap());

type Row = (i64, Option<Value>, Option<Value>, Option<i64>);

impl Db {
    /// This query is detailed in ``src/db/design/vote_registration_query.md``
    ///
    /// 'lower' is an optional inclusive lower bound. If `None`, `0` is used instead.
    /// 'upper' is an optional inclusive upper bound. If `None`, `i64::MAX` is used instead.
    ///
    /// Returns an error if either `lower` or `upper` is greater than `i64::MAX`
    ///
    /// # Errors
    ///
    /// Returns an error if either of `lower` or `upper` doesn't fit in an `i64`, or if a
    /// database-specific error occurs
    pub fn vote_registrations(
        &self,
        lower: SlotNo,
        upper: SlotNo,
    ) -> Result<Vec<SignedRegistration>> {
        let lower = lower.into_i64().ok_or_else(|| eyre!("invalid i64"))?;
        let upper = upper.into_i64().ok_or_else(|| eyre!("invalid i64"))?;
        let q = query(lower, upper);

        info!("Querying Registrations from DB from Slot# {lower} to {upper} Inclusive.");

        let rows = self.exec(move |conn| q.load(conn))?;

        let (oks, errs): (_, Vec<_>) = rows.into_iter().map(convert_row).partition_result();

        if errs.is_empty() {
            info!("No registration errors detected.");
        } else {
            let file = File::options()
                .write(true)
                .create(true)
                .open("convert_row.err")
                .unwrap();
            let mut writer = BufWriter::new(file);
            warn!("writing {} errors to convert_row.err", errs.len());
            for err in errs {
                writeln!(&mut writer, "==========================").unwrap();
                writeln!(&mut writer, "{err:#?}").unwrap();
            }
        }

        Ok(oks)
    }
}

/// This query doesn't exactly match the query in the doc. In particular, some filtering of invalid
/// JSONs isn't performed, since it breaks something (not sure what, just get no rows back). For
/// now, we'll just do the filtering in Rust
fn query(lower: i64, upper: i64) -> impl DbQuery<'static, Row> {
    let (meta_table, sig_table) = alias!(tx_metadata as meta_table, tx_metadata as sig_table);

    let metadata = meta_table.field(tx_metadata::json);
    let signature = sig_table.field(tx_metadata::json);

    let tables = meta_table
        .inner_join(tx::table.on(tx::id.eq(meta_table.field(tx_metadata::tx_id))))
        .inner_join(
            sig_table.on(sig_table
                .field(tx_metadata::tx_id)
                .eq(meta_table.field(tx_metadata::tx_id))),
        )
        .inner_join(block::table.on(block::id.eq(tx::block_id)));

    let selection = (
        meta_table.field(tx_metadata::tx_id),
        metadata,
        signature,
        block::slot_no,
    );

    let signature_keys_predicate = signature.has_all_keys(["1"].as_slice());
    let block_number_predicate = block::slot_no.ge(lower).and(block::slot_no.le(upper));

    let metadata_2 = metadata.retrieve_as_object("2");

    tables
        .filter(meta_table.field(tx_metadata::key).eq(&*METADATA_KEY))
        .filter(sig_table.field(tx_metadata::key).eq(&*SIGNATURE_KEY))
        .filter(signature_keys_predicate)
        .filter(block_number_predicate)
        .select(selection)
        .distinct_on(metadata_2)
}

/// Attempt to parse a row into a [`SignedRegistration`] struct
fn convert_row((tx_id, metadata, signature, _slot_no): Row) -> Result<SignedRegistration, Report> {
    let metadata = metadata.ok_or_else(|| eyre!("no metadata"))?;
    let signature = signature.ok_or_else(|| eyre!("no metadata"))?;

    let tx_id = u64::try_from(tx_id)?.into();
    let registration = serde_json::from_value(metadata.clone()).map_err(|e| {
        Report::from(e).with_warning(|| format!("registration json: {metadata:#?}"))
    })?;
    let signature = serde_json::from_value(signature.clone())
        .map_err(|e| Report::from(e).with_warning(|| format!("signature json: {signature:#?}")))?;

    Ok(SignedRegistration {
        registration,
        signature,
        tx_id,
    })
}

#[cfg(test)]
mod tests {
    use serde_json::{from_value, json};

    use crate::{
        data::{PubKey, Registration},
        VotingPowerSource,
    };

    use super::*;

    fn cip15_test_vector_meta() -> serde_json::Value {
        json!({
            "1": "0036ef3e1f0d3f5989e2d155ea54bdb2a72c4c456ccb959af4c94868f473f5a0",
            "2": "86870efc99c453a873a16492ce87738ec79a0ebd064379a62e2c9cf4e119219e",
            "3": "e0ae3a0a7aeda4aea522e74e4fe36759fca80789a613a58a4364f6ecef",
            "4": 1234,
        })
    }

    fn cip15_test_vector_sig() -> serde_json::Value {
        json!({
            "1": "6c2312cd49067ecf0920df7e067199c55b3faef4ec0bce1bd2cfb99793972478c45876af2bc271ac759c5ce40ace5a398b9fdb0e359f3c333fe856648804780e",
        })
    }

    // #[test]
    // fn process_accepts_cip15_test_vector() {
    //     let rows = vec![(
    //         1,
    //         Some(cip15_test_vector_meta()),
    //         Some(cip15_test_vector_sig()),
    //         None,
    //     )];
    //
    //     let regs = vec![SignedRegistration {
    //         tx_id: 1.into(),
    //         registration: from_value(cip15_test_vector_meta()).unwrap(),
    //         signature: from_value(cip15_test_vector_sig()).unwrap(),
    //     }];
    //
    //     let result: Vec<_> = rows.into_iter().filter_map(convert_row).collect();
    //
    //     assert_eq!(result, regs);
    // }

    #[test]
    fn filters_bad_rows() {
        fn check(row: Row) {
            assert!(convert_row(row).is_err());
        }

        // bad sig
        check((
            1,
            Some(cip15_test_vector_meta()),
            Some(json!("random json")),
            None,
        ));

        // bad meta
        check((
            1,
            Some(json!("random json")),
            Some(cip15_test_vector_sig()),
            None,
        ));

        // none
        check((1, None, None, None));
    }

    #[test]
    fn query_snapshot() {
        let q = query(123, 234);
        insta::assert_snapshot!(q.sql_string());
    }

    #[test]
    fn can_parse_voting_power_source() {
        color_eyre::install().unwrap();
        let vps = json!([[
            "0x1b92110d6c2aee00b7b208992efe3a2511f6811577155dd89977e8600fa5c27a",
            1
        ]]);
        let _: VotingPowerSource = from_value(vps).unwrap();
        let x = "0x0cf8d92c3fb4474942554a3c97f9a0421b69073fed4a600a6a77a1d72a8a";
        hex::decode(x.trim_start_matches("0x")).unwrap();

        let _: PubKey = from_value(json!(
            "0x0cf8d92c3fb4474942554a3c97f9a0421b69073fed4a600a6a77a1d72a8a"
        ))
        .unwrap();

        let value = json!({
            "1": [[
                "0x0cf8d92c3fb4474942554a3c97f9a0421b69073fed4a600a6a77a1d72a8a",
                1
            ]],
            "2": "0x19649e69c2fc0942d41452ad771278e0da18d1965bc9cc00d92d0e1a0f632ea4",
            "3": "0x005e868b277b468dcdec76f410fb9159e61565b3d61800dcfaab08afb7c17cbc9ffe7a69542d440e12fe800c708530e5a36c5f148c2e795f79",
            "4": 15_159_614,
            "5": 0,
        });
        let _: Registration = from_value(value).unwrap();
    }
}
