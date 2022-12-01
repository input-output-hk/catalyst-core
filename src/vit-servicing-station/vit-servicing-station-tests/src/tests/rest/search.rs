use crate::common::clients::SearchRequestBuilder;
use crate::common::startup::quick_start;
use assert_fs::TempDir;
use vit_servicing_station_lib::db::models::challenges::Challenge;
use vit_servicing_station_lib::db::models::proposals::FullProposalInfo;
use vit_servicing_station_lib::v0::endpoints::search::requests::{
    Column, SearchQuery, SearchResponse,
};

#[test]
pub fn search_challenges_by_title() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let rest_client = server.rest_client_with_token(&data.token_hash());
    let expected_challenge = &data.challenges()[0];

    let response = rest_client
        .search(
            SearchRequestBuilder::default()
                .on_challenges()
                .by_title(&expected_challenge.title)
                .into(),
        )
        .unwrap();

    assert_response_contains_challenge(expected_challenge, response);
}

fn assert_response_contains_challenge(
    expected_challenge: &Challenge,
    search_response: SearchResponse,
) {
    if let SearchResponse::Challenge(challenges) = search_response {
        assert!(challenges
            .iter()
            .map(|c| &c.title)
            .any(|title| *title == expected_challenge.title));
    } else {
        panic!("internal error: querying for challenges but got proposals");
    }
}

#[test]
pub fn search_challenges_by_type() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let rest_client = server.rest_client_with_token(&data.token_hash());
    let expected_challenge = &data.challenges()[0];

    let response = rest_client
        .search(
            SearchRequestBuilder::default()
                .on_challenges()
                .by_type(&expected_challenge.challenge_type)
                .into(),
        )
        .unwrap();

    assert_response_contains_challenge(expected_challenge, response);
}

#[test]
pub fn search_challenges_by_body() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let rest_client = server.rest_client_with_token(&data.token_hash());
    let expected_challenge = &data.challenges()[0];

    let response = rest_client
        .search(
            SearchRequestBuilder::default()
                .on_challenges()
                .by_body(&expected_challenge.description)
                .into(),
        )
        .unwrap();

    assert_response_contains_challenge(expected_challenge, response);
}

#[test]
pub fn search_challenges_by_title_empty() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let rest_client = server.rest_client_with_token(&data.token_hash());

    let response = rest_client
        .search(
            SearchRequestBuilder::default()
                .on_challenges()
                .by_title("aaaaaaaaaaaaaaaaaaaaaaaa")
                .into(),
        )
        .unwrap();

    if let SearchResponse::Challenge(challenges) = response {
        assert!(challenges.is_empty());
    } else {
        panic!("internal error: querying for challenges but got proposals");
    }
}

#[test]
pub fn search_proposal_by_author() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let rest_client = server.rest_client_with_token(&data.token_hash());
    let expected_proposal = &data.proposals()[0];

    let author_name = expected_proposal.proposal.proposer.proposer_name.clone();
    author_name.clone().remove(author_name.len() - 2);

    let response = rest_client
        .search(
            SearchRequestBuilder::default()
                .on_proposals()
                .by_author(author_name)
                .into(),
        )
        .unwrap();
    assert_response_contains_proposals(expected_proposal, response)
}

fn assert_response_contains_proposals(
    expected_proposal: &FullProposalInfo,
    search_response: SearchResponse,
) {
    if let SearchResponse::Proposal(proposals) = search_response {
        assert!(proposals
            .iter()
            .map(|c| &c.proposal.proposal_title)
            .any(|title| *title == expected_proposal.proposal.proposal_title));
    } else {
        panic!("internal error: querying for challenges but got proposals");
    }
}

#[test]
pub fn search_proposal_by_title() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let rest_client = server.rest_client_with_token(&data.token_hash());
    let expected_proposal = &data.proposals()[0];

    let response = rest_client
        .search(
            SearchRequestBuilder::default()
                .on_proposals()
                .by_title(&expected_proposal.proposal.proposal_title)
                .into(),
        )
        .unwrap();

    assert_response_contains_proposals(expected_proposal, response)
}

#[test]
pub fn search_proposal_by_funds() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let rest_client = server.rest_client_with_token(&data.token_hash());
    let expected_proposal = &data.proposals()[0];

    let response = rest_client
        .search(
            SearchRequestBuilder::default()
                .on_proposals()
                .by_funds_exact(expected_proposal.proposal.proposal_funds)
                .into(),
        )
        .unwrap();

    assert_response_contains_proposals(expected_proposal, response)
}

#[test]
pub fn search_proposal_by_funds_range() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let rest_client = server.rest_client_with_token(&data.token_hash());
    let expected_proposal = &data.proposals()[0];

    let response = rest_client
        .search(
            SearchRequestBuilder::default()
                .on_proposals()
                .by_funds(
                    Some(expected_proposal.proposal.proposal_funds - 10),
                    Some(expected_proposal.proposal.proposal_funds + 10),
                )
                .into(),
        )
        .unwrap();

    assert_response_contains_proposals(expected_proposal, response)
}

#[test]
pub fn search_proposal_by_title_and_author() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let rest_client = server.rest_client_with_token(&data.token_hash());
    let expected_proposal = &data.proposals()[0];

    let response = rest_client
        .search(
            SearchRequestBuilder::default()
                .on_proposals()
                .by_title(&expected_proposal.proposal.proposal_title)
                .by_author(&expected_proposal.proposal.proposer.proposer_name)
                .into(),
        )
        .unwrap();

    assert_response_contains_proposals(expected_proposal, response)
}

#[test]
pub fn search_proposal_by_funds_empty() {
    let temp_dir = TempDir::new().unwrap().into_persistent();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let rest_client = server.rest_client_with_token(&data.token_hash());

    let response = rest_client
        .search(
            SearchRequestBuilder::default()
                .on_proposals()
                .by_funds_exact(1)
                .into(),
        )
        .unwrap();

    assert!(response.is_empty());
}

#[test]
pub fn sort_challenges_result_by_title_desc() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let rest_client = server.rest_client_with_token(&data.token_hash());
    let filter_query = "a";
    let expected_challenges: Vec<Challenge> = data
        .challenges()
        .into_iter()
        .filter(|x| x.title.contains(filter_query))
        .collect();

    let response = rest_client
        .search(
            SearchRequestBuilder::default()
                .on_challenges()
                .by_title(filter_query)
                .order_by_desc(Column::Title)
                .into(),
        )
        .unwrap();

    if let SearchResponse::Challenge(challenges) = response {
        let mut expected: Vec<&String> = expected_challenges.iter().map(|x| &x.title).collect();
        expected.sort_by(|x, y| y.cmp(x));

        let actual: Vec<&String> = challenges.iter().map(|x| &x.title).collect();
        assert_eq!(expected, actual);
    } else {
        panic!("internal error: querying for challenges but got proposals");
    }
}

#[test]
pub fn sort_proposals_result_by_funds_asc() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let rest_client = server.rest_client_with_token(&data.token_hash());
    let filter_query = "a";
    let mut expected_proposals: Vec<FullProposalInfo> = data
        .proposals()
        .into_iter()
        .filter(|x| x.proposal.proposal_title.contains(filter_query))
        .collect();
    expected_proposals.sort_by(|x, y| x.proposal.proposal_funds.cmp(&y.proposal.proposal_funds));

    let response = rest_client
        .search(
            SearchRequestBuilder::default()
                .on_proposals()
                .by_title(filter_query)
                .order_by_asc(Column::Funds)
                .into(),
        )
        .unwrap();

    if let SearchResponse::Proposal(proposals) = response {
        let expected: Vec<String> = expected_proposals
            .iter()
            .map(|x| x.proposal.proposal_funds.to_string())
            .collect();
        let actual: Vec<String> = proposals
            .iter()
            .map(|x| x.proposal.proposal_funds.to_string())
            .collect();
        assert_eq!(expected, actual);
    } else {
        panic!("internal error: querying for proposals but got challenges");
    }
}

#[test]
pub fn sort_proposals_result_by_title_desc() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let rest_client = server.rest_client_with_token(&data.token_hash());
    let filter_query = "a";
    let expected_proposals: Vec<FullProposalInfo> = data
        .proposals()
        .into_iter()
        .filter(|x| x.proposal.proposal_title.contains(filter_query))
        .collect();
    let response = rest_client
        .search(
            SearchRequestBuilder::default()
                .on_proposals()
                .by_title(filter_query)
                .order_by_desc(Column::Title)
                .into(),
        )
        .unwrap();

    if let SearchResponse::Proposal(proposals) = response {
        let mut expected: Vec<&String> = expected_proposals
            .iter()
            .map(|x| &x.proposal.proposal_title)
            .collect();
        expected.sort();
        expected.reverse();
        let actual: Vec<&String> = proposals
            .iter()
            .map(|x| &x.proposal.proposal_title)
            .collect();
        assert_eq!(expected, actual);
    } else {
        panic!("internal error: querying for proposals but got challenges");
    }
}

#[test]
pub fn search_proposal_by_impact_score() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let rest_client = server.rest_client_with_token(&data.token_hash());

    let expected_proposal = &data.proposals()[0];
    let impact_score = data.proposals()[0].proposal.proposal_impact_score;

    let response = rest_client
        .search(
            SearchRequestBuilder::default()
                .on_proposals()
                .by_impact_score(impact_score)
                .into(),
        )
        .unwrap();

    assert_response_contains_proposals(expected_proposal, response)
}
#[test]
pub fn sort_proposals_result_by_title_random() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let rest_client = server.rest_client_with_token(&data.token_hash());
    let filter_query = "a";
    let expected_proposals: Vec<FullProposalInfo> = data
        .proposals()
        .into_iter()
        .filter(|x| x.proposal.proposal_title.contains(filter_query))
        .collect();

    let search_query: SearchQuery = SearchRequestBuilder::default()
        .on_proposals()
        .by_title(filter_query)
        .order_by_random()
        .into();

    let response = rest_client.search(search_query.clone()).unwrap();

    let first_random_proposals = if let SearchResponse::Proposal(proposals) = response {
        assert_eq!(expected_proposals.len(), proposals.len());
        assert_ne!(expected_proposals, proposals);
        proposals
    } else {
        panic!("internal error: querying for proposals but got challenges");
    };

    let response = rest_client.search(search_query).unwrap();

    if let SearchResponse::Proposal(proposals) = response {
        assert_eq!(first_random_proposals.len(), proposals.len());
        assert_ne!(first_random_proposals, proposals);
    } else {
        panic!("internal error: querying for proposals but got challenges");
    }
}

#[test]
pub fn search_proposals_limit() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let rest_client = server.rest_client_with_token(&data.token_hash());
    let filter_query = "a";
    let mut expected_proposals: Vec<FullProposalInfo> = data
        .proposals()
        .into_iter()
        .filter(|x| x.proposal.proposal_title.contains(filter_query))
        .collect();

    expected_proposals.sort_by(|x, y| y.proposal.proposal_title.cmp(&x.proposal.proposal_title));

    let response = rest_client
        .search(
            SearchRequestBuilder::default()
                .on_proposals()
                .by_title(filter_query)
                .order_by_desc(Column::Title)
                .limit(1)
                .into(),
        )
        .unwrap();

    println!(
        "{:?}",
        expected_proposals
            .iter()
            .map(|x| &x.proposal.proposal_title)
            .collect::<Vec<&String>>()
    );

    if let SearchResponse::Proposal(proposals) = response {
        let expected = vec![expected_proposals[0].proposal.proposal_title.clone()];
        let actual: Vec<String> = proposals
            .iter()
            .map(|x| x.proposal.proposal_title.clone())
            .collect();
        assert_eq!(expected, actual);
    } else {
        panic!("internal error: querying for proposals but got challenges");
    }
}

#[test]
pub fn search_proposals_offset() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let rest_client = server.rest_client_with_token(&data.token_hash());
    let filter_query = "a";
    let mut expected_proposals: Vec<FullProposalInfo> = data
        .proposals()
        .into_iter()
        .filter(|x| x.proposal.proposal_title.contains(filter_query))
        .collect();
    expected_proposals.sort_by(|x, y| y.proposal.proposal_title.cmp(&x.proposal.proposal_title));

    let response = rest_client
        .search(
            SearchRequestBuilder::default()
                .on_proposals()
                .by_title(filter_query)
                .order_by_desc(Column::Title)
                .offset(1)
                .limit(1)
                .into(),
        )
        .unwrap();

    if let SearchResponse::Proposal(proposals) = response {
        let expected = vec![expected_proposals[1].proposal.proposal_title.clone()];
        let actual: Vec<String> = proposals
            .iter()
            .map(|x| x.proposal.proposal_title.clone())
            .collect();
        assert_eq!(expected, actual);
    } else {
        panic!("internal error: querying for proposals but got challenges");
    }
}
