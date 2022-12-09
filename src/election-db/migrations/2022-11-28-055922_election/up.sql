-- Catalyst Fund Metadata Database

-- Version of the schema.


CREATE VIEW schema_version AS SELECT
    version
FROM __diesel_schema_migrations
WHERE version = (SELECT MAX(version) FROM __diesel_schema_migrations);


-- Config Table
-- This table is looked up with three keys, `id`, `id2` and `id3`

CREATE TABLE config
(
  row_id SERIAL PRIMARY KEY,
  id     VARCHAR NOT NULL,
  id2    VARCHAR NOT NULL,
  id3    VARCHAR NOT NULL,
  value  JSONB NULL
);

-- id+id2+id3 must be unique, they are a combined key.
CREATE UNIQUE INDEX config_idx ON config(id,id2,id3);


COMMENT ON TABLE config IS
'General JSON Configuration and Data Values.
Defined  Data Formats:
  Schema Version:
    `id` = "version"
    `id2` = "" (Unused)
    `id3` = "" (Unused)
    `value`->"version" = "<Version string of the API".
    NOTE: We may be able to remove this if we can use the diesel api version directly.
  API Tokens:
    `id` = "api_token"
    `id2` = <API Token, encrypted with a secret, as base-64 encoded string "">`
    `id3` = "" (Unused),
    `value`->"name" = "<Name of the token owner>",
    `value`->"created" = <Integer Unix Epoch when Token was created>,
    `value`->"expires" = <Integer Unix Epoch when Token will expire>,
';
COMMENT ON COLUMN config.row_id IS 'Synthetic unique key.  Always lookup using id.';
COMMENT ON COLUMN config.id IS  'The name/id of the general config value/variable';
COMMENT ON COLUMN config.id2 IS '2nd ID of the general config value. Must be defined, use "" if not required.';
COMMENT ON COLUMN config.id3 IS '3rd ID of the general config value. Must be defined, use "" if not required.';
COMMENT ON COLUMN config.value IS 'The JSON value of the system variable id.id2.id3';

COMMENT ON INDEX config_idx IS 'We use three keys combined uniquely rather than forcing string concatenation at the app level to allow for querying groups of data.';

-- Elections Table - Defines each election

CREATE TABLE election
(
    row_id SERIAL PRIMARY KEY,

    name TEXT NOT NULL,
    description TEXT NOT NULL,

    registration_snapshot_time TIMESTAMP,
    voting_power_threshold BIGINT,
    max_voting_power_pct NUMERIC(6,3) CONSTRAINT percentage CHECK (max_voting_power_pct <= 100),

    start_time TIMESTAMP,
    end_time TIMESTAMP,

    insight_sharing_start TIMESTAMP,
    proposal_submission_start TIMESTAMP,
    refine_proposals_start TIMESTAMP,
    finalize_proposals_start TIMESTAMP,
    proposal_assessment_start TIMESTAMP,
    assessment_qa_start TIMESTAMP,
    snapshot_start TIMESTAMP,
    voting_start TIMESTAMP,
    voting_end TIMESTAMP,
    tallying_end TIMESTAMP,

    extra JSONB
);

CREATE UNIQUE INDEX election_name_idx ON election(name);

COMMENT ON TABLE election IS 'The basic parameters of each election.';
COMMENT ON COLUMN election.row_id IS '';
COMMENT ON COLUMN election.name IS 'The name of the election, for example "Fund9" or "SVE1"';
COMMENT ON COLUMN election.description IS 'A detailed description of the purpose of the election.  For example, the Funds "Goal".';
COMMENT ON COLUMN election.registration_snapshot_time IS 'The Time (UTC) Registrations are taken from Cardano main net,  registrations after this date are not valid for voting on the event. NULL = Not yet defined or Not Applicable.';
COMMENT ON COLUMN election.voting_power_threshold IS 'The Minimum number of Lovelace staked at the time of snapshot, to be eligible to vote. NULL = Not yet defined.';
COMMENT ON COLUMN election.start_time IS 'The time (UTC) the election starts.  NULL = Not yet defined.';
COMMENT ON COLUMN election.end_time IS 'The time (UTC) the election ends.  NULL = Not yet defined.';
COMMENT ON COLUMN election.insight_sharing_start IS 'TODO';
COMMENT ON COLUMN election.proposal_submission_start IS 'The Time (UTC) proposals can start to be submitted for the Election.  NULL = Not yet defined, or Not applicable.';
COMMENT ON COLUMN election.refine_proposals_start IS 'TODO';
COMMENT ON COLUMN election.finalize_proposals_start IS 'The Time (UTC) when all proposals must be finalized by. NULL = Not yet defined, or Not applicable.';
COMMENT ON COLUMN election.proposal_assessment_start IS 'The Time (UTC) when PA Assessors can start assessing proposals. NULL = Not yet defined, or Not applicable.';
COMMENT ON COLUMN election.assessment_qa_start IS 'The Time (UTC) when vPA Assessors can start assessing assessments. NULL = Not yet defined, or Not applicable.';
COMMENT ON COLUMN election.snapshot_start IS 'The Time (UTC) when the voting power of registered stake addresses is taken from main net. NULL = Not yet defined.';
COMMENT ON COLUMN election.voting_start IS 'The earliest time that registered wallets with sufficient voting power can place votes in the election. NULL = Not yet defined.';
COMMENT ON COLUMN election.voting_end IS 'The latest time that registered wallets with sufficient voting power can place votes in the election. NULL = Not yet defined.';
COMMENT ON COLUMN election.tallying_end IS 'The latest time that tallying the election can complete by. NULL = Not yet defined.';
COMMENT ON COLUMN election.extra IS 'Json Map defining election specific extra data. NULL = Not yet defined. "url"."results" = a results URL, "url"."survey" = a survey URL, others can be defined as required.';

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
    ('ADA','Cardano ADA.');

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


-- challenge table - Defines all challenges for all known funds.


CREATE TABLE challenge
(
    row_id SERIAL PRIMARY KEY,

    id INTEGER NOT NULL,
    election INTEGER NOT NULL,

    category TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,

    rewards_currency TEXT,
    rewards_total BIGINT,
    proposers_rewards BIGINT,
    vote_options INTEGER,

    extra JSONB,

    FOREIGN KEY(election) REFERENCES election(row_id),
    FOREIGN KEY(category) REFERENCES challenge_category(name),
    FOREIGN KEY(rewards_currency) REFERENCES currency(name),
    FOREIGN KEY(vote_options) REFERENCES vote_options(id)
);

CREATE UNIQUE INDEX challenge_idx ON challenge (id, election);

COMMENT ON TABLE challenge IS 'All Challenges for all elections. A Challenge is a group category for selection in an election.';
COMMENT ON COLUMN challenge.row_id IS 'Synthetic Unique Key';
COMMENT ON COLUMN challenge.id IS 'Fund specific Challenge ID, can be non-unique between funds (Eg, Ideascale ID for challenge).';
COMMENT ON COLUMN challenge.election IS 'The specific Election ID this Challenge is part of.';
COMMENT ON COLUMN challenge.category IS 'What category of challenge is this, see the challenge_category table for allowed values.';
COMMENT ON COLUMN challenge.title IS 'The  title of the challenge.';
COMMENT ON COLUMN challenge.description IS 'Long form description of the challenge.';
COMMENT ON COLUMN challenge.rewards_currency IS 'The currency rewards values are represented as.';
COMMENT ON COLUMN challenge.rewards_total IS 'The total reward pool to pay on this challenge to winning proposals.';
COMMENT ON COLUMN challenge.proposers_rewards IS 'Not sure how this is different from rewards_total???';
COMMENT ON COLUMN challenge.vote_options IS 'The Vote Options applicable to all proposals in this challenge.';
COMMENT ON COLUMN challenge.extra IS 'Extra Data  for this challenge represented as JSON.  "url"."challenge" is a URL for more info about the challenge.  "highlights" is ???';

-- Proposals Table

CREATE TABLE proposal
(
    row_id SERIAL PRIMARY KEY,
    id INTEGER NOT NULL UNIQUE,
    challenge INTEGER NOT NULL,
    title TEXT NOT NULL,
    summary TEXT NOT NULL,
    public_key TEXT NOT NULL,
    funds BIGINT NOT NULL,
    url TEXT NOT NULL,
    files_url TEXT NOT NULL,
    impact_score BIGINT NOT NULL,

    extra JSONB,

    proposer_name TEXT NOT NULL,
    proposer_contact TEXT NOT NULL,
    proposer_url TEXT NOT NULL,
    proposer_relevant_experience TEXT NOT NULL,
    bb_proposal_id BYTEA,

    bb_vote_options TEXT,

    FOREIGN KEY(challenge) REFERENCES challenge(row_id),
    FOREIGN KEY(bb_vote_options) REFERENCES vote_options(challenge)
);

COMMENT ON TABLE proposal IS 'All Proposals for the current fund.';
COMMENT ON COLUMN proposal.row_id IS 'Synthetic Unique Key';
COMMENT ON COLUMN proposal.id IS 'Actual Proposal Unique ID';
COMMENT ON COLUMN proposal.challenge IS 'The Challenge this proposal falls under.';
COMMENT ON COLUMN proposal.title IS 'Brief title of the proposal.';
COMMENT ON COLUMN proposal.summary IS 'A Summary of the proposal to be implemented.';
COMMENT ON COLUMN proposal.public_key IS '???';
COMMENT ON COLUMN proposal.funds IS 'How much funds (in the currency of the fund)';
COMMENT ON COLUMN proposal.url IS 'A URL with supporting information for the proposal.';
COMMENT ON COLUMN proposal.files_url IS 'A URL link to relevant files supporting the proposal.';
COMMENT ON COLUMN proposal.impact_score IS 'The Impact score assigned to this proposal by the Assessors.';
COMMENT ON COLUMN proposal.proposer_name IS 'The proposers name.';
COMMENT ON COLUMN proposal.proposer_contact IS 'Contact details for the proposer.';
COMMENT ON COLUMN proposal.proposer_url IS 'A URL with details of the proposer.';
COMMENT ON COLUMN proposal.proposer_relevant_experience IS 'A freeform  string describing the proposers experience relating to their capability to implement the proposal.';
COMMENT ON COLUMN proposal.bb_proposal_id IS 'The ID used by the voting ledger (bulletin board) to refer to this proposal.';
COMMENT ON COLUMN proposal.bb_vote_options IS 'The selectable options by the voter.';
COMMENT ON COLUMN proposal.extra IS
'Extra data about the proposal.
 The types of extra data are defined by the proposal type and are not enforced.
 Extra Fields for `native` challenges:
    NONE.

 Extra Fields for `simple` challenges:
    "solution" : <text> - The Solution to the challenge.

 Extra Fields for `community choice` challenge:
    "brief"      : <text> - Brief explanation of a proposal.
    "importance" : <text> - The importance of the proposal.
    "goal"       : <text> - The goal of the proposal is addressed to meet.
    "metrics"    : <text> - The metrics of the proposal or how success will be determined.';

-- Vote Plan Categories

CREATE TABLE voteplan_category
(
    name TEXT PRIMARY KEY,
    public_key BOOL
);


INSERT INTO voteplan_category (name, public_key)
VALUES
    ('public', false), -- Fully public votes only
    ('private', true), -- Fully private votes only.
    ('semi-private', true); -- Private until tally, then decrypted.

COMMENT ON TABLE voteplan_category IS 'The category of vote plan currently supported.';
COMMENT ON COLUMN voteplan_category.name IS 'The UNIQUE name of this voteplan category.';
COMMENT ON COLUMN voteplan_category.public_key IS 'Does this vote plan category require a public key.';


-- groups

CREATE TABLE voting_group (
    row_id SERIAL PRIMARY KEY,
    group_id VARCHAR NOT NULL,
    election_id INTEGER NOT NULL,
    token_id VARCHAR,

    FOREIGN KEY(election_id) REFERENCES election(row_id)
);

CREATE UNIQUE INDEX token_fund_id ON voting_group (token_id, election_id);

COMMENT ON TABLE voting_group IS 'All Groups.';
COMMENT ON COLUMN voting_group.row_id IS 'Synthetic Unique Key.';
COMMENT ON COLUMN voting_group.group_id IS 'The ID of this voting group.';
COMMENT ON COLUMN voting_group.election_id IS 'The Election this voting group belongs to.';
COMMENT ON COLUMN voting_group.token_id IS 'The ID of the voting token used by this group.';

-- Vote Plans

CREATE TABLE voteplan
(
    row_id SERIAL PRIMARY KEY,
    election_id INTEGER NOT NULL,

    id VARCHAR NOT NULL UNIQUE,
    category TEXT NOT NULL,
    encryption_key VARCHAR,
    group_id INTEGER,

    FOREIGN KEY(election_id) REFERENCES election(row_id),
    FOREIGN KEY(category) REFERENCES voteplan_category(name),
    FOREIGN KEY(group_id) REFERENCES voting_group(row_id)
);

COMMENT ON TABLE voteplan IS 'All Vote plans.';

COMMENT ON COLUMN voteplan.row_id IS 'Synthetic Unique Key';
COMMENT ON COLUMN voteplan.id IS 'The ID of the Vote plan in the voting ledger/bulletin board. A Binary value encoded as hex.';
COMMENT ON COLUMN voteplan.category IS 'The kind of vote which can be cast on this vote plan.';
COMMENT ON COLUMN voteplan.encryption_key IS 'The public encryption key used, ONLY if required by the voteplan category.';
COMMENT ON COLUMN voteplan.group_id IS 'The identifier of voting power token used withing this plan.';

-- Table to link Proposals to Vote plans in a many-many relationship.
-- This Many-Many relationship arises because:
--  in the vote ledger/bulletin board,
--      one proposal may be within multiple different vote plans,
--      and each voteplan can contain multiple proposals.
CREATE TABLE proposal_voteplan
(
    row_id SERIAL PRIMARY KEY,
    proposal_id INTEGER,
    voteplan_id INTEGER,
    bb_proposal_index BIGINT,

    FOREIGN KEY(proposal_id) REFERENCES proposal(row_id) ON DELETE CASCADE,
    FOREIGN KEY(voteplan_id) REFERENCES voteplan(row_id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX proposal_voteplan_idx ON proposal_voteplan(proposal_id,voteplan_id,bb_proposal_index);

COMMENT ON TABLE proposal_voteplan IS 'Table to link Proposals to Vote plans in a Many to Many relationship.';
COMMENT ON COLUMN proposal_voteplan.row_id IS 'Synthetic ID of this Voteplan/Proposal M-M relationship.';
COMMENT ON COLUMN proposal_voteplan.proposal_id IS 'The link to the Proposal primary key that links to this voteplan.';
COMMENT ON COLUMN proposal_voteplan.voteplan_id IS 'The link to the Voteplan primary key that links to this proposal.';
COMMENT ON COLUMN proposal_voteplan.bb_proposal_index IS 'The Index with the voteplan used by the voting ledger/bulletin board that references this proposal.';

-- community advisor reviews

-- I feel like these ratings and notes should be in a  general json field to
-- suit adaptability without needing schema changes.

CREATE TABLE community_advisors_review (
  row_id SERIAL PRIMARY KEY,
  proposal_id INTEGER NOT NULL,
  assessor VARCHAR NOT NULL,
  impact_alignment_rating_given INTEGER,
  impact_alignment_note VARCHAR,
  feasibility_rating_given INTEGER,
  feasibility_note VARCHAR,
  auditability_rating_given INTEGER,
  auditability_note VARCHAR,
  ranking INTEGER,

  FOREIGN KEY (proposal_id) REFERENCES proposal(id) ON DELETE CASCADE
);

COMMENT ON TABLE community_advisors_review IS 'All Reviews.';
COMMENT ON COLUMN community_advisors_review.row_id IS 'Synthetic Unique Key.';
COMMENT ON COLUMN community_advisors_review.proposal_id IS 'The Proposal this review is for.';
COMMENT ON COLUMN community_advisors_review.assessor IS 'Assessors Anonymized ID';
COMMENT ON COLUMN community_advisors_review.impact_alignment_rating_given IS 'The  numeric rating assigned to the proposal by the assessor.';
COMMENT ON COLUMN community_advisors_review.impact_alignment_note IS 'A note about why the impact rating was given.';
COMMENT ON COLUMN community_advisors_review.feasibility_rating_given IS 'The numeric feasibility rating given.';
COMMENT ON COLUMN community_advisors_review.feasibility_note IS 'A note about why the feasibility rating was given.';
COMMENT ON COLUMN community_advisors_review.auditability_rating_given IS 'The numeric auditability rating given.';
COMMENT ON COLUMN community_advisors_review.auditability_note IS 'A note about the auditability rating given.';
COMMENT ON COLUMN community_advisors_review.ranking IS 'Numeric  Measure of quality of this review according to veteran community advisors.';

-- goals

CREATE TABLE goal
(
    id SERIAL PRIMARY KEY,
    election_id INTEGER NOT NULL,

    idx integer NOT NULL,
    name VARCHAR NOT NULL,

    FOREIGN KEY(election_id) REFERENCES election(row_id)
);

CREATE UNIQUE INDEX goal_index ON goal(election_id, idx);

COMMENT ON TABLE goal IS 'The list of campaign goals for this election.';
COMMENT ON COLUMN goal.id IS 'Synthetic Unique Key.';
COMMENT ON COLUMN goal.idx IS 'The index specifying the order/priority of the goals.';
COMMENT ON COLUMN goal.name IS 'The description of this election goal.';
COMMENT ON COLUMN goal.election_id IS 'The ID of the election this goal belongs to.';
COMMENT ON INDEX goal_index IS 'An index to enforce uniqueness of the relative `idx` field per election.';

-- categories of voter registration

CREATE TABLE voter_registration_category(
    name TEXT PRIMARY KEY,
    description TEXT
);

INSERT INTO voter_registration_category (name, description)
VALUES
    ('cip15','cip-15 registration'), -- CIP-15 style registration.
    ('cip36','cip-36 registration'), -- CIP-36 style registration.
    ('cip36d', 'cip-36 deregistration'); -- CIP-36 style deregistration.

-- Individual voter registrations

CREATE TABLE voter_registration (
    row_id SERIAL8 PRIMARY KEY,

    time TIMESTAMP,
    nonce BIGINT,

    purpose BIGINT,
    stake_pub TEXT,

    category TEXT,
    delegations JSONB,
    reward_addr TEXT,

    txn BYTEA,
    block BIGINT,

    FOREIGN KEY(category) REFERENCES voter_registration_category(name)
);

CREATE UNIQUE INDEX latest_registration_index ON voter_registration(time,nonce,purpose,stake_pub);

COMMENT ON TABLE voter_registration IS 'Every voter registration made on cardano.';
COMMENT ON COLUMN voter_registration.row_id IS 'Synthetic Unique ID for a particular voter registration record.';
COMMENT ON COLUMN voter_registration.time IS 'The time the voter registration was made, derived from the block the registration was in.';
COMMENT ON COLUMN voter_registration.delegations IS
'The JSON equivalent encoding of the voting key or delegations Array:
  IF category == "cip15":
    {"key":"<voting key>"}
    (See cip-15 <https://cips.cardano.org/cips/cip15/> for details.)
  IF category == "cip36":
    {"delegation":[["<voting key>",weight],...]}
    (See cip-36 <https://cips.cardano.org/cips/cip36/> for details.)
  IF category == "cip36d":
    {}
    (See cip-36 <https://cips.cardano.org/cips/cip36/> for details.)';
COMMENT ON COLUMN voter_registration.stake_pub IS 'A stake address for the network that this transaction is submitted to (to point to the Ada that is being delegated).';
COMMENT ON COLUMN voter_registration.reward_addr IS 'A Shelley address discriminated for the same network this transaction is submitted to to receive rewards.';
COMMENT ON COLUMN voter_registration.nonce IS 'A nonce that identifies that most recent delegation. In case multiple registrations make it into the same block.';
COMMENT ON COLUMN voter_registration.purpose IS 'A non-negative integer that indicates the purpose of the vote.';
COMMENT ON COLUMN voter_registration.txn IS 'The raw transaction from the blockchain, including its witness.';
COMMENT ON COLUMN voter_registration.block IS 'The block this transaction is located in on the cardano blockchain.';
COMMENT ON INDEX latest_registration_index IS 'Index to find latest registrations per unique stake_pub key faster.';

-- Every balance change to every stake address, ever.

CREATE TABLE stake_address_balance (
    row_id SERIAL8 PRIMARY KEY,
    time TIMESTAMP,
    block  BIGINT,

    public_key TEXT,
    balance NUMERIC,
    unpaid_rewards NUMERIC
);

COMMENT ON TABLE stake_address_balance IS 'The balance of a particular stake address at a particular point in time. Note, this table catches ALL stake addresses, not only those registered.';
COMMENT ON COLUMN stake_address_balance.row_id IS 'Synthetic record ID of the balance update.';
COMMENT ON COLUMN stake_address_balance.time IS 'The time the stake address balance changed to the value in this record.';
COMMENT ON COLUMN stake_address_balance.block IS
'The block number on cardano where this balance change occurred.
If there were multiple changes to the stake address balance made in the same block, then this record is the result of
all changes at the end of the block, after all transaction are processed.';
COMMENT ON COLUMN stake_address_balance.public_key IS 'The stake public key address who''s value changed.';
COMMENT ON COLUMN stake_address_balance.balance IS 'The ADA.LOVELACE balance of the stake address, at this time.';
COMMENT ON COLUMN stake_address_balance.unpaid_rewards IS 'The ADA.LOVELACE balance of all unpaid rewards associated with this public stake key address.';

-- snapshot - Etched in stone snapshot of voting power for each election.

CREATE TABLE voting_power (
    row_id SERIAL8 PRIMARY KEY,
    election INTEGER,

    voting_key TEXT,
    power NUMERIC,

    FOREIGN KEY(election) REFERENCES Election(row_id)
)


/*

-- votes

CREATE TABLE votes (
    fragment_id TEXT PRIMARY KEY,

    caster TEXT NOT NULL,
    proposal INTEGER NOT NULL,
    voteplan_id TEXT NOT NULL,
    time TIMESTAMP NOT NULL,
    choice SMALLINT,
    raw_fragment BYTEA NOT NULL
);

-- TODO, Shouldn't this be related to the other tables?

COMMENT ON TABLE votes IS 'All Votes.';
COMMENT ON COLUMN votes.fragment_id is 'TODO';
COMMENT ON COLUMN votes.caster is 'TODO';
COMMENT ON COLUMN votes.proposal is 'TODO';
COMMENT ON COLUMN votes.voteplan_id is 'TODO';
COMMENT ON COLUMN votes.time is 'TODO';
COMMENT ON COLUMN votes.choice is 'TODO';
COMMENT ON COLUMN votes.raw_fragment is 'TODO';

-- snapshots

CREATE TABLE snapshots (
    tag TEXT PRIMARY KEY,
    last_updated TIMESTAMP NOT NULL
);

COMMENT ON TABLE snapshots IS 'TODO';
COMMENT ON COLUMN snapshots.last_updated is 'TODO';

-- voters

CREATE TABLE voters (
    row_id SERIAL PRIMARY KEY,

    voting_key TEXT NOT NULL,
    voting_group TEXT NOT NULL,
    snapshot_tag TEXT NOT NULL,

    voting_power BIGINT NOT NULL,

    FOREIGN KEY(snapshot_tag) REFERENCES snapshots(tag) ON DELETE CASCADE
);

CREATE UNIQUE INDEX unique_voter_id on voters (voting_key, voting_group, snapshot_tag);

-- TODO, Shouldn't this be related to the other tables, like groups?

COMMENT ON TABLE voters IS 'TODO';
COMMENT ON COLUMN voters.row_id is 'Synthetic Unique Row Key';
COMMENT ON COLUMN voters.voting_key is 'TODO';
COMMENT ON COLUMN voters.voting_power is 'TODO';
COMMENT ON COLUMN voters.voting_group is 'TODO';
COMMENT ON COLUMN voters.snapshot_tag is 'TODO';



-- contributions

CREATE TABLE contributions (
    row_id SERIAL PRIMARY KEY,

    stake_public_key TEXT NOT NULL,
    voting_key TEXT NOT NULL,
    voting_group TEXT NOT NULL,
    snapshot_tag TEXT NOT NULL,

    reward_address TEXT NOT NULL,
    value BIGINT NOT NULL,

    FOREIGN KEY(snapshot_tag) REFERENCES snapshots(tag) ON DELETE CASCADE
);

CREATE UNIQUE INDEX unique_contribution_id ON contributions (stake_public_key, voting_key, voting_group, snapshot_tag);

COMMENT ON TABLE contributions IS 'TODO';
COMMENT ON COLUMN contributions.row_id is 'Synthetic Unique Row Key';
COMMENT ON COLUMN contributions.stake_public_key IS 'TODO';
COMMENT ON COLUMN contributions.voting_key IS 'TODO';
COMMENT ON COLUMN contributions.voting_group IS 'TODO';
COMMENT ON COLUMN contributions.snapshot_tag IS 'TODO';
COMMENT ON COLUMN contributions.reward_address IS 'TODO';
COMMENT ON COLUMN contributions.value IS 'TODO';
*/