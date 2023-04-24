INSERT INTO
    proposal (
        row_id,
        id,
        objective,
        title,
        summary,
        category,
        public_key,
        funds,
        url,
        files_url,
        impact_score,
        extra,
        proposer_name,
        proposer_contact,
        proposer_url,
        proposer_relevant_experience,
        bb_proposal_id
    )
VALUES (
        1,
        1,
        1,
        'title 1',
        'summary 1',
        'category 1',
        'b7a3c12dc0c8c748ab07525b701122b88bd78f600c76342d27f25e5f92444cde',
        100,
        'url.xyz',
        'files.xyz',
        555,
        '{"brief": "Brief explanation of a proposal", "importance": "The importance of the proposal", "goal": "The goal of the proposal is addressed to meet"}',
        'alice',
        'alice@io',
        'alice.prop.xyz',
        'alice is an attorney',
        '70726f706f73616c'
    );

INSERT INTO
    proposal (
        row_id,
        id,
        objective,
        title,
        summary,
        category,
        public_key,
        funds,
        url,
        files_url,
        impact_score,
        extra,
        proposer_name,
        proposer_contact,
        proposer_url,
        proposer_relevant_experience,
        bb_proposal_id
    )
VALUES (
        2,
        2,
        2,
        'title 2',
        'summary 2',
        'category 2',
        'b7a3c12dc0c8c748ab07525b701122b88bd78f600c76342d27f25e5f92444cde',
        100,
        'url.xyz',
        'files.xyz',
        555,
        '{"brief": "Brief explanation of a proposal", "importance": "The importance of the proposal", "goal": "The goal of the proposal is addressed to meet"}',
        'bob',
        'bob@io',
        'bob.prop.xyz',
        'bob is an accountant',
        '70726f706f73616c'
    );