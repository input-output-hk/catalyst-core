-- Catalyst Event Database

-- Subscription Table
-- This table stores the subscriptions for users in the reviews module

CREATE TABLE subscription
(
  row_id    SERIAL PRIMARY KEY,
  user_id   INTEGER NOT NULL,
  event_id  INTEGER NOT NULL,
  role      INTEGER NOT NULL,
  status    INTEGER NOT NULL,
  extra     JSONB NULL,

  FOREIGN KEY (user_id) REFERENCES catalyst_user(row_id) ON DELETE CASCADE,
  FOREIGN KEY (event_id) REFERENCES event(row_id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX subscription_idx ON subscription(user_id,event_id,role);

COMMENT ON TABLE subscription IS '
A subscription describes the possible roles of a single user for an event.
A user can participate in multiple events active as multiple roles.
For this reason this dedicated table tracks all active roles for a user related
to a specific event and stores meta information about them.
Some of these subscriptions will be automatically created by the system.
The presence/status of subscriptions will determine user capabilities in the app.
';

COMMENT ON COLUMN subscription.user_id IS 'The user ID this subscription belongs to.';
COMMENT ON COLUMN subscription.event_id IS 'The event ID this subscription belongs to.';

COMMENT ON COLUMN subscription.role IS
'This field describes the role of the user for this subscription.
Possible values:
0: For LV0 Community Reviewers
1: For LV1 Community Reviewers
2: For LV2 Moderators';

COMMENT ON COLUMN subscription.extra IS
'This field is used to store all meta information about a subscription.
Specifically:

anonymous_id: str,
subscription_date: datetime,
preferred_categories: [int],
reward_address: str
';


-- Allocation - Defines the relationship between users and proposals or proposals_reviews
-- to describe:
-- - the ownership of a proposal.
-- - the allocation of the reviews that needs to be done.
-- - the allocation of moderations that needs to be done.

CREATE TABLE allocation (
  row_id      SERIAL PRIMARY KEY,
  proposal_id INTEGER NULL,
  review_id   INTEGER NULL,
  user_id     INTEGER NOT NULL,
  type        INTEGER NOT NULL,

  FOREIGN KEY (proposal_id) REFERENCES proposal(row_id) ON DELETE CASCADE,
  FOREIGN KEY (review_id) REFERENCES proposal_review(row_id) ON DELETE CASCADE,
  FOREIGN KEY (user_id) REFERENCES catalyst_user(row_id) ON DELETE CASCADE
);


COMMENT ON TABLE allocation IS 'The relationship between users and proposals or proposals_reviews.';
COMMENT ON COLUMN allocation.row_id IS 'Synthetic ID of this relationship.';
COMMENT ON COLUMN allocation.proposal_id IS 'The proposal ID the relationship belongs to.';
COMMENT ON COLUMN allocation.review_id IS 'The review ID the relationship belongs to.';
COMMENT ON COLUMN allocation.user_id IS 'The user ID the relationship belongs to.';
COMMENT ON COLUMN allocation.type IS 'The type of relationship stored.
Possible values:
0: proposal ownership relation. proposal_id and user_id are required
1: proposal allocated for review. proposal_id and user_id are required
2: review allocated for moderation. review_id and user_id are required';

CREATE INDEX idx_allocation_proposal_type ON allocation(proposal_id, type) WHERE proposal_id IS NOT NULL;
CREATE INDEX idx_allocation_review_type ON allocation(review_id, type) WHERE review_id IS NOT NULL;
CREATE INDEX idx_allocation_user_type ON allocation(user_id, type);


-- Moderation - Defines the moderation submitted by users for each proposal_review.

CREATE TABLE moderation (
  row_id          SERIAL PRIMARY KEY,
  review_id       INTEGER NOT NULL,
  user_id         INTEGER NOT NULL,
  classification  INTEGER NOT NULL,
  rationale       VARCHAR,
  UNIQUE (review_id, user_id),

  FOREIGN KEY (review_id) REFERENCES proposal_review(row_id) ON DELETE CASCADE,
  FOREIGN KEY (user_id) REFERENCES catalyst_user(row_id) ON DELETE CASCADE
);


COMMENT ON TABLE moderation IS 'An individual moderation for a proposal review.';
COMMENT ON COLUMN moderation.row_id IS 'Synthetic ID of this moderation.';
COMMENT ON COLUMN moderation.review_id IS 'The review ID the moderation belongs to.';
COMMENT ON COLUMN moderation.user_id IS 'The user ID the moderation belongs to.';
COMMENT ON COLUMN moderation.classification IS 'The value used to describe the moderation (e.g. 0: excluded, 1: included).';
COMMENT ON COLUMN moderation.rationale IS 'The rationale for the given classification.';