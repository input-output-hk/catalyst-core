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

fn challenge_field_error() -> Rejection {
    make_error("`challenge` doesn't support `funds` or `author`", 400)
}

fn proposal_field_error() -> Rejection {
    make_error("`proposal` doesn't support `type`", 400)
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
            if filter.iter().any(|c| {
                matches!(
                    c,
                    Constraint::Text {
                        column: Column::Funds | Column::Author,
                        ..
                    }
                )
            }) {
                return Err(challenge_field_error());
            }

            if matches!(
                order_by,
                Some(OrderBy::Column {
                    column: Column::Funds | Column::Author,
                    ..
                })
            ) {
                return Err(challenge_field_error());
            }

            let ctx = context.read().unwrap();
            let challenges = ctx.state().vit().challenges();
            let result = search_challenges(challenges, &filter, order_by);
            let result = limit_and_offset(result, limit, offset);
            Ok(SearchResponse::Challenge(result))
        }
        Table::Proposals => {
            if filter.iter().any(|c| {
                matches!(
                    c,
                    Constraint::Text {
                        column: Column::Type,
                        ..
                    }
                )
            }) {
                return Err(proposal_field_error());
            }

            if filter.iter().any(|c| {
                matches!(
                    c,
                    Constraint::Text {
                        column: Column::Funds,
                        ..
                    }
                )
            }) {
                return Err(proposal_field_error());
            }

            if matches!(
                order_by,
                Some(OrderBy::Column {
                    column: Column::Type,
                    ..
                })
            ) {
                return Err(make_error("can't search proposal by `fund`", 400));
            }

            let ctx = context.read().unwrap();
            let proposals = ctx.state().vit().proposals();
            let result = search_proposals(proposals, &filter, order_by);
            let result = limit_and_offset(result, limit, offset);
            Ok(SearchResponse::Proposal(result))
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

fn search_proposals(
    mut proposals: Vec<FullProposalInfo>,
    filter: &[Constraint],
    order_by: Option<OrderBy>,
) -> Vec<FullProposalInfo> {
    fn is_match(proposal: &FullProposalInfo, constraint: &Constraint) -> bool {
        let Constraint::Text { column, search } = constraint else { todo!() };
        let string = match column {
            Column::Desc => &proposal.proposal.proposal_summary,
            Column::Title => &proposal.proposal.proposal_title,
            Column::Author => &proposal.proposal.proposer.proposer_name,
            _ => return false,
        };

        string.to_lowercase().contains(&search.to_lowercase())
    }

    let should_retain = |p: &FullProposalInfo| filter.iter().all(|cons| is_match(p, cons));
    proposals.retain(should_retain);

    if let Some(OrderBy::Column { column, descending }) = order_by {
        match column {
            Column::Desc => proposals.sort_by(|a, b| {
                a.proposal
                    .proposal_summary
                    .cmp(&b.proposal.proposal_summary)
            }),
            Column::Title => {
                proposals.sort_by(|a, b| a.proposal.proposal_title.cmp(&b.proposal.proposal_title))
            }
            Column::Funds => {
                proposals.sort_by(|a, b| a.proposal.proposal_funds.cmp(&b.proposal.proposal_funds))
            }
            Column::Author => proposals.sort_by(|a, b| {
                a.proposal
                    .proposer
                    .proposer_name
                    .cmp(&b.proposal.proposer.proposer_name)
            }),
            _ => {}
        };

        if descending {
            proposals.reverse();
        }
    }

    proposals
}

fn search_challenges(
    mut challenges: Vec<Challenge>,
    filter: &[Constraint],
    order_by: Option<OrderBy>,
) -> Vec<Challenge> {
    fn is_match(challenge: &Challenge, constraint: &Constraint) -> bool {
        let Constraint::Text { search, column } = constraint else { todo!() };
        let string = match column {
            Column::Type => {
                return challenge
                    .challenge_type
                    .to_string()
                    .to_lowercase()
                    .contains(&challenge.challenge_type.to_string())
            }
            Column::Desc => &challenge.description,
            Column::Title => &challenge.title,
            _ => return false,
        };

        string.to_lowercase().contains(&search.to_lowercase())
    }

    let should_retain = |c: &Challenge| filter.iter().all(|cons| is_match(c, cons));
    challenges.retain(should_retain);

    if let Some(OrderBy::Column { column, descending }) = order_by {
        match column {
            Column::Type => challenges.sort_by(|a, b| {
                a.challenge_type
                    .to_string()
                    .cmp(&b.challenge_type.to_string())
            }),
            Column::Desc => challenges.sort_by(|a, b| a.description.cmp(&b.description)),
            Column::Title => challenges.sort_by(|a, b| a.title.cmp(&b.title)),
            _ => {}
        }

        if descending {
            challenges.reverse();
        }
    }

    challenges
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
