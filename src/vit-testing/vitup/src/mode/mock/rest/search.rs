use rand::seq::SliceRandom;
use rand::thread_rng;
use vit_servicing_station_lib::db::models::proposals::FullProposalInfo;
use vit_servicing_station_lib::v0::endpoints::search::requests::*;
use vit_servicing_station_lib::{db::models::challenges::Challenge, v0::result::HandlerResult};
use warp::{Rejection, Reply};

use super::reject::GeneralException;
use crate::mode::mock::ContextLock;

fn make_error(s: &str, code: u16) -> Rejection {
    warp::reject::custom(GeneralException {
        summary: s.to_string(),
        code,
    })
}

fn mock_order_by_error() -> Rejection {
    make_error("Mock implementation only supports 0 or 1 `order_by`s", 400)
}

#[tracing::instrument(skip(context), name = "REST Api call")]
pub async fn search(
    search_query: SearchQuery,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    let response = search_impl(search_query, context).await?;
    Ok(HandlerResult(Ok(response)))
}

#[tracing::instrument(skip(context), name = "REST Api call")]
pub async fn search_count(
    search_query_count: SearchCountQuery,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    let search_query = SearchQuery {
        query: search_query_count,
        limit: None,
        offset: None,
    };

    let response = search_impl(search_query, context).await?;

    Ok(HandlerResult(Ok(match response {
        SearchResponse::Challenge(challenges) => challenges.len(),
        SearchResponse::Proposal(proposals) => proposals.len(),
    })))
}

async fn search_impl(
    SearchQuery {
        query:
            SearchCountQuery {
                table,
                filter,
                order_by,
            },
        limit,
        offset,
    }: SearchQuery,
    context: ContextLock,
) -> Result<SearchResponse, Rejection> {
    let order_by = match order_by[..] {
        [] => None,
        [order] => Some(order),
        _ => return Err(mock_order_by_error()),
    };
    match table {
        Table::Challenges => {
            let mut results = context.read().unwrap().state().vit().challenges();

            for f in &filter {
                use Column::*;
                match f {
                    Constraint::Text { search, column } => {
                        let search = search.to_lowercase();
                        let string_function = match column {
                            Type => |c: &Challenge| c.challenge_type.to_string(),
                            Title => |c: &Challenge| c.title.clone(),
                            Desc => |c: &Challenge| c.description.clone(),
                            Author | Funds | ImpactScore => {
                                return Err(make_error("invalid column", 400))
                            }
                        };

                        results.retain(|c| string_function(c).to_lowercase().contains(&search))
                    }
                    Constraint::Range {
                        lower,
                        upper,
                        column,
                    } => {
                        let lower = lower.unwrap_or(i64::MIN);
                        let upper = upper.unwrap_or(i64::MAX);

                        let num_function = match column {
                            Funds => |c: &Challenge| c.rewards_total,
                            _ => return Err(make_error("invalid column", 400)),
                        };

                        results.retain(|c| {
                            let num = num_function(c);
                            lower <= num && num <= upper
                        });
                    }
                }

                if let Some(OrderBy::Column { column, descending }) = order_by {
                    match column {
                        Column::Type => results.sort_by(|a, b| {
                            a.challenge_type
                                .to_string()
                                .cmp(&b.challenge_type.to_string())
                        }),
                        Column::Desc => results.sort_by(|a, b| a.description.cmp(&b.description)),
                        Column::Title => results.sort_by(|a, b| a.title.cmp(&b.title)),
                        _ => {}
                    }

                    if descending {
                        results.reverse();
                    }
                }
            }
            let results = limit_and_offset(results, limit, offset);
            Ok(SearchResponse::Challenge(results))
        }
        Table::Proposals => {
            let mut results = context.read().unwrap().state().vit().proposals();

            for f in &filter {
                use Column::*;
                match f {
                    Constraint::Text { search, column } => {
                        let search = search.to_lowercase();
                        let string_function = match column {
                            Type => |p: &FullProposalInfo| p.challenge_type.to_string(),
                            Title => |p: &FullProposalInfo| p.proposal.proposal_title.clone(),
                            Desc => |p: &FullProposalInfo| p.proposal.proposal_summary.clone(),
                            Author => {
                                |p: &FullProposalInfo| p.proposal.proposer.proposer_name.clone()
                            }
                            Funds | ImpactScore => return Err(make_error("invalid column", 400)),
                        };

                        results.retain(|p| string_function(p).to_lowercase().contains(&search))
                    }
                    Constraint::Range {
                        lower,
                        upper,
                        column,
                    } => {
                        let lower = lower.unwrap_or(i64::MIN);
                        let upper = upper.unwrap_or(i64::MAX);

                        let num_function = match column {
                            Funds => |p: &FullProposalInfo| p.proposal.proposal_funds,
                            ImpactScore => |p: &FullProposalInfo| p.proposal.proposal_impact_score,
                            _ => return Err(make_error("invalid column", 400)),
                        };

                        results.retain(|p| {
                            let num = num_function(p);
                            lower <= num && num <= upper
                        });
                    }
                }

                match order_by {
                    None => {}
                    Some(OrderBy::Random) => results.shuffle(&mut thread_rng()),
                    Some(OrderBy::Column { column, descending }) => {
                        match column {
                            Column::Desc => results.sort_by(|a, b| {
                                a.proposal
                                    .proposal_summary
                                    .cmp(&b.proposal.proposal_summary)
                            }),
                            Column::Title => results.sort_by(|a, b| {
                                a.proposal.proposal_title.cmp(&b.proposal.proposal_title)
                            }),
                            Column::Funds => results.sort_by(|a, b| {
                                a.proposal.proposal_funds.cmp(&b.proposal.proposal_funds)
                            }),
                            Column::Author => results.sort_by(|a, b| {
                                a.proposal
                                    .proposer
                                    .proposer_name
                                    .cmp(&b.proposal.proposer.proposer_name)
                            }),
                            _ => {}
                        };

                        if descending {
                            results.reverse();
                        }
                    }
                }
            }

            let results = limit_and_offset(results, limit, offset);
            Ok(SearchResponse::Proposal(results))
        }
    }
}

fn limit_and_offset<T>(vec: Vec<T>, limit: Option<u64>, offset: Option<u64>) -> Vec<T> {
    let offset = offset.unwrap_or(0);
    let limit = limit.unwrap_or(u64::MAX);
    vec.into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::make_context;

    use super::*;

    #[test]
    fn limit_and_offset_works_properly() {
        let vec = vec![1, 2, 3, 4, 5, 6];
        let after = limit_and_offset(vec, Some(2), Some(1));
        assert_eq!(after, vec![2, 3]);
    }

    #[tokio::test]
    async fn can_seach_all_challenges() {
        let context = make_context!();

        let query = SearchQuery {
            query: SearchCountQuery {
                table: Table::Challenges,
                filter: vec![],
                order_by: vec![],
            },
            limit: None,
            offset: None,
        };

        let result = search_impl(query, context.clone()).await.unwrap();
        let mut challenges = match result {
            SearchResponse::Proposal(_) => panic!(),
            SearchResponse::Challenge(challenges) => challenges,
        };

        let mut expected_challenges = context.read().unwrap().state().vit().challenges();

        challenges.sort_by_key(|c| c.internal_id);
        expected_challenges.sort_by_key(|c| c.internal_id);

        assert_eq!(challenges, expected_challenges);
    }

    #[tokio::test]
    async fn can_seach_some_challenges() {
        let context = make_context!();

        let query = SearchQuery {
            query: SearchCountQuery {
                table: Table::Challenges,
                filter: vec![Constraint::Text {
                    search: "1".to_string(),
                    column: Column::Desc,
                }],
                order_by: vec![],
            },
            limit: None,
            offset: None,
        };

        let result = search_impl(query, context.clone()).await.unwrap();
        let challenges = match result {
            SearchResponse::Proposal(_) => panic!(),
            SearchResponse::Challenge(challenges) => challenges,
        };

        let challenge = context
            .read()
            .unwrap()
            .state()
            .vit()
            .challenges()
            .into_iter()
            .find(|c| c.description.contains('1'))
            .unwrap();

        assert_eq!(challenges, vec![challenge]);
    }
}
