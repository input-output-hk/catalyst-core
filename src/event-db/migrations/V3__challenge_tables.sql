-- Catalyst Event Database

-- challenge types table - Defines all currently known challenges types.
CREATE TABLE challenge_category
(
    name TEXT PRIMARY KEY,
    description TEXT
);

COMMENT ON TABLE challenge_category IS 'Defines all known and valid challenge categories.';
COMMENT ON COLUMN challenge_category.name IS 'The name of this challenge category.';
COMMENT ON COLUMN challenge_category.description IS 'A Description of this kind of challenge category.';

-- Define known challenge categories
INSERT INTO challenge_category (name,  description)
VALUES
    ('simple','A Simple choice'),
    ('native','??'),
    ('community-choice','Community collective decision');

-- known currencies - Defines all currently known currencies.
CREATE TABLE currency
(
    name TEXT PRIMARY KEY,
    description TEXT
);

COMMENT ON TABLE currency IS 'Defines all known and valid currencies.';
COMMENT ON COLUMN currency.name IS 'The name of this currency type.';
COMMENT ON COLUMN currency.description IS 'A Description of this kind of currency type.';


-- Define known currencies
INSERT INTO currency (name,  description)
VALUES
    ('USD_ADA','US Dollars, converted to Cardano ADA at time of reward calculation.'),
    ('ADA','Cardano ADA.'),
    ('CLAP', 'CLAP tokens.'),
    ('COTI', 'COTI tokens.');

-- known vote options - Defines all currently known vote options.
CREATE TABLE vote_options
(
    id SERIAL PRIMARY KEY,
    idea_scale TEXT UNIQUE,
    challenge TEXT UNIQUE
);

COMMENT ON TABLE vote_options IS 'Defines all known vote plan option types.';
COMMENT ON COLUMN vote_options.id IS 'Unique ID for each possible option set.';
COMMENT ON COLUMN vote_options.idea_scale IS 'How this vote option is represented in idea scale.';
COMMENT ON COLUMN vote_options.challenge IS 'How the vote options is represented in the challenge.';

-- Define known vote_options
INSERT INTO vote_options (idea_scale,  challenge)
VALUES
    ('blank,yes,no','yes,no');



-- goals

CREATE TABLE goal
(
    id SERIAL PRIMARY KEY,
    event_id INTEGER NOT NULL,

    idx integer NOT NULL,
    name VARCHAR NOT NULL,

    FOREIGN KEY(event_id) REFERENCES event(row_id)
);

CREATE UNIQUE INDEX goal_index ON goal(event_id, idx);

COMMENT ON TABLE goal IS 'The list of campaign goals for this event.';
COMMENT ON COLUMN goal.id IS 'Synthetic Unique Key.';
COMMENT ON COLUMN goal.idx IS 'The index specifying the order/priority of the goals.';
COMMENT ON COLUMN goal.name IS 'The description of this event goal.';
COMMENT ON COLUMN goal.event_id IS 'The ID of the event this goal belongs to.';
COMMENT ON INDEX goal_index IS 'An index to enforce uniqueness of the relative `idx` field per event.';


-- challenge table - Defines all challenges for all known funds.


CREATE TABLE challenge
(
    row_id SERIAL PRIMARY KEY,

    id INTEGER NOT NULL,
    event INTEGER NOT NULL,

    category TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,

    rewards_currency TEXT,
    rewards_total BIGINT,
    proposers_rewards BIGINT,
    vote_options INTEGER,

    extra JSONB,

    FOREIGN KEY(event) REFERENCES event(row_id),
    FOREIGN KEY(category) REFERENCES challenge_category(name),
    FOREIGN KEY(rewards_currency) REFERENCES currency(name),
    FOREIGN KEY(vote_options) REFERENCES vote_options(id)
);

CREATE UNIQUE INDEX challenge_idx ON challenge (id, event);
CREATE UNIQUE INDEX challenge_id ON challenge (id);

COMMENT ON TABLE challenge IS
'All Challenges for all events.
A Challenge is a group category for selection in an event.';
COMMENT ON COLUMN challenge.row_id IS 'Synthetic Unique Key';
COMMENT ON COLUMN challenge.id IS
'Event specific Challenge ID.
Can be non-unique between events (Eg, Ideascale ID for challenge).';
COMMENT ON COLUMN challenge.event IS 'The specific Event ID this Challenge is part of.';
COMMENT ON COLUMN challenge.category IS
'What category of challenge is this.
See the challenge_category table for allowed values.';
COMMENT ON COLUMN challenge.title IS 'The  title of the challenge.';
COMMENT ON COLUMN challenge.description IS 'Long form description of the challenge.';
COMMENT ON COLUMN challenge.rewards_currency IS 'The currency rewards values are represented as.';
COMMENT ON COLUMN challenge.rewards_total IS 'The total reward pool to pay on this challenge to winning proposals.';
COMMENT ON COLUMN challenge.proposers_rewards IS 'Not sure how this is different from rewards_total???';
COMMENT ON COLUMN challenge.vote_options IS 'The Vote Options applicable to all proposals in this challenge.';
COMMENT ON COLUMN challenge.extra IS
'Extra Data  for this challenge represented as JSON.
"url"."challenge" is a URL for more info about the challenge.
"highlights" is ???
';
