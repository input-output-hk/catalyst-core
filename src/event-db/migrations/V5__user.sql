-- Catalyst Event Database

-- User Table
-- This table stores the users for the reviews module

CREATE TABLE catalyst_user
(
  row_id        SERIAL PRIMARY KEY,
  catalyst_id   INTEGER NOT NULL,
  enc_password  VARCHAR NOT NULL,
  salt          VARCHAR NOT NULL,
  extra         JSONB NULL,

  FOREIGN KEY(catalyst_id) REFERENCES catalyst_id_map(row_id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX user_catalyst_id_idx ON catalyst_user(catalyst_id);


COMMENT ON TABLE catalyst_user IS '
This tables stores the user account for the Review Module.
It contains a reference to the catalyst_id that is used as public and unique identifier.
';

COMMENT ON COLUMN catalyst_user.catalyst_id IS 'The catalyst_id this account belongs to.';
COMMENT ON COLUMN catalyst_user.enc_password IS 'The encrypted password.';
COMMENT ON COLUMN catalyst_user.salt IS 'The salt for the password encryption.';
COMMENT ON COLUMN catalyst_user.extra IS
'This field is used to store all meta information about a user that 
are required by the application. Specifically:

admin: bool,
username: str,
registration_date: datetime,
due_diligence: {
  status: int,
  due_diligence_id: str,
  url: str
},
historic_stats: {
  "reviews": {
    "active_funds": int,
    "submitted": int,
    "blank": int,
    "valid": int
  },
  "moderations": {
    "active_funds": int,
    "submitted": int,
    "allocated": int,
  }
}
';
