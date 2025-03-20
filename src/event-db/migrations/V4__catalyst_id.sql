-- Catalyst Event Database

-- Catalyst ID map Table
-- This table stores the relationship between a catalyst ID and an email address.

CREATE TABLE catalyst_id_map
(
  row_id SERIAL PRIMARY KEY,
  catalyst_id VARCHAR NOT NULL,
  email VARCHAR NOT NULL,
  signed_email  VARCHAR NOT NULL,
  status  INTEGER NOT NULL,
  confirmation_token VARCHAR NOT NULL, 

  UNIQUE(catalyst_id)
);

CREATE UNIQUE INDEX catalyst_id_idx ON catalyst_id_map(catalyst_id, email);

COMMENT ON COLUMN catalyst_id_map.email IS 'The email is stored encrypted.';
COMMENT ON COLUMN catalyst_id_map.status IS '
Describes the status of an account:
0: Inactive
1: Active
2: Banned.';