INSERT INTO objective
(row_id, id, event, category, title, description, rewards_currency, rewards_total, proposers_rewards, vote_options, extra)
VALUES 
(1,
1, 1,
'catalyst-simple', 'title 1', 'description 1',
'ADA', 100, 100, 1,
'{"url": "objective 1 url", "sponsor": "objective 1 sponsor", "video": "objective 1 video"}'
);

INSERT INTO objective
(row_id, id, event, category, title, description)
VALUES 
(2,
2, 1,
'catalyst-native', 'title 2', 'description 2'
);
