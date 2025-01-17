-- Catalyst Event Database

-- Catalyst ID map Table
-- This table stores the relationship between a catalyst ID and an email address.

CREATE TABLE catalyst_id_map
(
  row_id                    SERIAL PRIMARY KEY,
  catalyst_id               VARCHAR NOT NULL,
  email                     VARCHAR NOT NULL,
  username                  VARCHAR NOT NULL,
  signed_email              VARCHAR NOT NULL,
  status                    INTEGER NOT NULL,
  confirmation_token        VARCHAR, 
  confirmation_requested_at TIMESTAMP,
  confirmed_at              TIMESTAMP,

  UNIQUE(catalyst_id)
);

CREATE UNIQUE INDEX catalyst_id_idx ON catalyst_id_map(catalyst_id, email);

COMMENT ON TABLE catalyst_id_map IS '
The `catalyst_id_map` table is used to store the map between the CatalystID and email/username for each user.
Because the Catalyst RBAC registration are stored on-chain, and for this reason are publicly accessible,
sensitive information like email address will be stored in a centralized DB, keeping a reference to the CatalystID.
The email address is stored only when its signature corresponds to the CatalystID.
';
COMMENT ON COLUMN catalyst_id_map.catalyst_id IS '
It contains the unique part of the CatalystID.
Given a full CatalystID like `id.catalyst://fred@preprod.cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE`
The scheme and the username are omitted and only the unique part `preprod.cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE`
is stored in this field.
';
COMMENT ON COLUMN catalyst_id_map.email IS 'The email address stored in plaintext.';
COMMENT ON COLUMN catalyst_id_map.username IS 'The username extracted from the CatalystID stored in plaintext.';
COMMENT ON COLUMN catalyst_id_map.signed_email IS 'The signed document that includes the email address.';
COMMENT ON COLUMN catalyst_id_map.status IS '
Describes the status of an account:
0: Inactive
1: Active
2: Banned.';
COMMENT ON COLUMN catalyst_id_map.confirmation_token IS 'The token that is generated for the email confirmation.';
COMMENT ON COLUMN catalyst_id_map.confirmation_requested_at IS 'The timestamp to validate confirmation token validity.';
COMMENT ON COLUMN catalyst_id_map.confirmed_at IS 'The timestamp of the email confirmation.';