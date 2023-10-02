#[cfg(test)]
mod test {
    use jormungandr_lib::crypto::account::Identifier;
    use tracing::Level;
    use voting_hir::VoterHIR;
    use warp::hyper::StatusCode;
    use warp::{Filter, Reply};

    async fn get_voting_power<F>(
        tag: &str,
        voting_key: &str,
        filter: &F,
    ) -> Result<Vec<(u64, String)>, StatusCode>
    where
        F: Filter + 'static,
        F::Extract: Reply + Send,
    {
        let result = warp::test::request()
            .path(format!("/snapshot/{}/{}", tag, voting_key).as_ref())
            .reply(filter)
            .await;

        let status = result.status();
        if !matches!(status, StatusCode::OK) {
            return Err(status);
        }

        let result_voting_power: Vec<serde_json::Value> =
            serde_json::from_str(dbg!(&String::from_utf8(result.body().to_vec()).unwrap()))
                .unwrap();

        Ok(result_voting_power
            .into_iter()
            .map(|v| {
                (
                    v["voting_power"].as_u64().unwrap(),
                    v["voting_group"].as_str().unwrap().to_string(),
                )
            })
            .collect::<Vec<_>>())
    }

    #[tokio::test]
    async fn test_snapshot_get_tags() {
        const GROUP1: &str = "group1";
        const GROUP2: &str = "group2";

        let _e = tracing_subscriber::fmt()
            .with_max_level(Level::TRACE)
            .with_writer(tracing_subscriber::fmt::TestWriter::new())
            .try_init();

        let keys = [
            "0000000000000000000000000000000000000000000000000000000000000000",
            "1111111111111111111111111111111111111111111111111111111111111111",
        ];

        let content_a = serde_json::to_string(&[
            VoterHIR {
                voting_key: Identifier::from_hex(keys[0]).unwrap(),
                voting_group: GROUP1.to_string(),
                voting_power: 1.into(),
            },
            VoterHIR {
                voting_key: Identifier::from_hex(keys[0]).unwrap(),
                voting_group: GROUP2.to_string(),
                voting_power: 2.into(),
            },
        ])
        .unwrap();

        let content_b = serde_json::to_string(&[VoterHIR {
            voting_key: Identifier::from_hex(keys[0]).unwrap(),
            voting_group: GROUP1.to_string(),
            voting_power: 2.into(),
        }])
        .unwrap();

        let (shared_context, update_handler) = snapshot_service::new_context().unwrap();

        let snapshot_root = warp::path!("snapshot" / ..).boxed();
        let filter = snapshot_service::filter(snapshot_root.clone(), shared_context.clone());
        let put_filter = snapshot_root.and(snapshot_service::update_filter(update_handler));

        assert_eq!(
            warp::test::request()
                .path(format!("/snapshot/{}", "tag_a").as_ref())
                .method("PUT")
                .body(content_a)
                .reply(&put_filter)
                .await
                .status(),
            StatusCode::OK
        );

        assert_eq!(
            warp::test::request()
                .path(format!("/snapshot/{}", "tag_b").as_ref())
                .method("PUT")
                .body(content_b)
                .reply(&put_filter)
                .await
                .status(),
            StatusCode::OK
        );

        assert_eq!(
            get_voting_power("tag_a", keys[0], &filter).await.unwrap(),
            vec![(1u64, GROUP1.to_string()), (2u64, GROUP2.to_string())]
        );

        assert_eq!(
            get_voting_power("tag_b", keys[0], &filter).await.unwrap(),
            vec![(2u64, GROUP1.to_string())]
        );

        assert!(get_voting_power("tag_c", keys[0], &filter).await.is_err());

        let result = warp::test::request().path("/snapshot").reply(&filter).await;

        let status = result.status();
        if !matches!(status, StatusCode::OK) {
            todo!();
        }

        let mut tags: Vec<String> =
            serde_json::from_str(&String::from_utf8(result.body().to_vec()).unwrap()).unwrap();

        tags.sort_unstable();

        assert_eq!(tags, vec!["tag_a", "tag_b"]);
    }
}
