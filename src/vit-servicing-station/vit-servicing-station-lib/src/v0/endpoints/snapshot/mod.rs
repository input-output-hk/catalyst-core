#[cfg(test)]
mod test {
    use jormungandr_lib::crypto::account::Identifier;
    use snapshot_lib::{KeyContribution, SnapshotInfo, VoterHIR};
    use tracing::Level;
    use warp::hyper::StatusCode;
    use warp::{Filter, Reply};

    async fn get_voters_info<F>(
        tag: &str,
        voting_key: &str,
        filter: &F,
    ) -> Result<Vec<(u64, u64, u64, String)>, StatusCode>
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

        let req_body = String::from_utf8(result.body().to_vec()).unwrap();
        let snapshot_service::VotersInfo {
            voter_info,
            last_updated: _timestamp,
        } = serde_json::from_str(&req_body).unwrap();

        Ok(voter_info
            .into_iter()
            .map(|v| {
                (
                    u64::from(v.voting_power),
                    v.delegations_count,
                    v.delegations_power,
                    v.voting_group,
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

        let content_a = serde_json::to_string(&snapshot_service::SnapshotInfoUpdate {
            snapshot: vec![
                SnapshotInfo {
                    contributions: vec![
                        KeyContribution {
                            reward_address: "address_1".to_string(),
                            value: 2,
                        },
                        KeyContribution {
                            reward_address: "address_2".to_string(),
                            value: 2,
                        },
                    ],
                    hir: VoterHIR {
                        voting_key: Identifier::from_hex(keys[0]).unwrap(),
                        voting_group: GROUP1.to_string(),
                        voting_power: 1.into(),
                    },
                },
                SnapshotInfo {
                    contributions: vec![KeyContribution {
                        reward_address: "address_3".to_string(),
                        value: 3,
                    }],
                    hir: VoterHIR {
                        voting_key: Identifier::from_hex(keys[0]).unwrap(),
                        voting_group: GROUP2.to_string(),
                        voting_power: 2.into(),
                    },
                },
            ],
            update_timestamp: 0,
        })
        .unwrap();

        let content_b = serde_json::to_string(&snapshot_service::SnapshotInfoUpdate {
            snapshot: vec![SnapshotInfo {
                contributions: vec![],
                hir: VoterHIR {
                    voting_key: Identifier::from_hex(keys[0]).unwrap(),
                    voting_group: GROUP1.to_string(),
                    voting_power: 2.into(),
                },
            }],
            update_timestamp: 1,
        })
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
            get_voters_info("tag_a", keys[0], &filter).await.unwrap(),
            vec![
                (1u64, 2u64, 4u64, GROUP1.to_string()),
                (2u64, 1u64, 3u64, GROUP2.to_string())
            ]
        );

        assert_eq!(
            get_voters_info("tag_b", keys[0], &filter).await.unwrap(),
            vec![(2u64, 0u64, 0u64, GROUP1.to_string())]
        );

        assert!(get_voters_info("tag_c", keys[0], &filter).await.is_err());

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
