# Registrations Database

There will be a Registration Database.
This is a time-series database, which means that updates do not replace old records, they are time-stamped instead.
This allows for the state "at a particular time" to be recovered without recreating it.

## Data

The data needed to be stored in each registration record is:

* The time and date the registration was made.
  * Derived from the block date/time on Cardano, NOT the time it was detected.
* The location of the transaction on the blockchain.
  * Allows the transaction to be verified against the blockchain.
* The raw contents of the transaction.
  * The full raw transaction in binary.
  * Allows information not directly pertinent to Catalyst to be retained.
* The Type of registration.
  * CIP-15
  * CIP-36
  * Others
    * Currently, there are only CIP-15 and CIP-36 voter registrations, however, there WILL be others.
* Invalidity Report
  * Is the registration transaction Valid according to Catalyst transaction validity rules?
    * true - Then this field is NULL.
    * false - then this field contains a JSON formatted report detailing WHY it was invalid.
* Registration specific fields
  * Fields which represent the meta-data of the registration itself.
  * These fields need to be able to be efficiently searched.

## Queries

Examples of Common Queries:

### Current Voter Registration

Given:

* `AsAt` - The time the registration must be valid by.
* `Window` - Optional, the maximum age of the registration before `AsAt` (Defaults to forever).
* `Stake Address` - the Stake Address.
* `Valid` - Must the registration be valid? (Tristate: True, False, None)
  * True - Only return valid registrations.
  * False - Only return invalid registrations IF there is no newer valid registration.
  * None - Return the most recent registration, Valid or not.

Return the MOST current registration.

### All Current Voter Registrations

Given:

* `AsAt` - The time the registration must be valid by.
* `Window` - Optional, the maximum age of the registration before `AsAt` (Defaults to forever).
* `Valid` - Must the registration be valid? (Tristate: True, False, None)
  * True - Only return valid registrations.
  * False - Only return invalid registrations IF there is no newer valid registration.
  * None - Return the most recent registration, Valid or not.

For each unique stake address:
    Return the MOST current registration.
