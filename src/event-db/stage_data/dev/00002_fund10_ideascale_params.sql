-- Define F10 IdeaScale parameters.
INSERT INTO config (id, id2, id3, value) VALUES (
    'ideascale',
    'params',
    'F10',
    '{
        "campaign_group_id": 63,
        "stage_ids": [4590, 4596, 4602, 4608, 4614, 4620, 4626, 4632, 4638, 4644, 4650, 4656, 4662]
     }'
);

-- Use F10 params for event with row_id = 10.
INSERT INTO config (id, id2, id3, value) VALUES (
    'event',
    'ideascale_params',
    '10',
    '{"params_id": "F10"}'
);
