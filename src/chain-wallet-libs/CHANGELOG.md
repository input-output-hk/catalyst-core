# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## [0.8.2]
- Updated Javascript wallet bindings, initial version of CIP-62 specification API.
- *breaking change*: wallet_spending_counter replaced by wallet_spending_counters
- *breaking change*: wallet_set_state now takes an array of spending counters 
- *breaking change*: wallet_vote now takes a lane as an argument
- *breaking change*: wallet_import_keys now takes a single argument with the
  account key.

## [0.8.0-alpha1]

- Remove the keygen crate due to lack of need for it.
- Remove wallet-jni and use wallet-uniffi instead.

## [0.7.0-pre4] 2021-09-02

- Bech32 HRP for encryption key now is set by chain-libs

## [0.7.0-pre3] 2021-08-27

- Fix compilation issue in iOS for cordova

## [0.7.0-pre2] 2021-08-26

### Changed

- Improved documentation
- Recover from mnemonics can now take a password

#### Wallet-js

- *deprecated*: VotePlanId::new_from_bytes
- *deprecated*: Ed25519Signature::from_binary
- *deprecated*: FragmentId::new_from_bytes

### Added

#### Wallet-js

- VotePlanId::from_bytes
- Ed25519Signature::from_bytes
- FragmentId::from_bytes

## [0.7.0-pre1] 2021-08-25

### Changed

- *breaking change*: `wallet_vote_cast` and `wallet_convert` now require the TTL argument.
- *breaking change*: `settings_new` takes new arguments needed for managing time.

### Added

- max_expiration_date
- block_date_from_system_time

## [0.6.0-pre2] 2021-05-14

### Added

#### cordova - c - java

- spending_counter
- settings_new
- settings_get
- fragment_from_raw
- fragment_id
- fragment_delete

### Changed

pending_transactions now returns transactions in order relative to the same
wallet type instead of arbitrary order.  First starting with deadalus/yoroi/free
utxo keys (those are all exclusive) in order of creating, and then the account
transactions, also in order of creation (and signing).

## [0.6.0-pre1]

### Changed

#### cordova - c - java - wallet-js 

- *breaking change*: Take a bech32 string in proposal_new_private instead of raw bytes

## [0.5.0] - 2021-01-05

## [0.5.0-pre9] - 2020-12-17

- Add explicit link to libdl.so in release process

## [0.5.0-pre8] - 2020-12-04

#### wallet-js

- Remove `symmetric_encrypt` (moved to *keygen*)
- Remove `symmetric_decrypt` (moved to *keygen*)
- Remove `bech32_decode_to_bytes` (moved to *keygen*)

#### keygen

Add specific package used for daedalus catalyst

*Features*

- Generate Ed25519Extended private/public keys.
- Encrypt keys with pin.
- Decode bech32 strings.

## [0.5.0-pre7] - 2020-11-16

#### wallet-js

- Expose function to decode bech32 to byte array

## [0.5.0-pre6] - 2020-11-06

### Changed

#### wallet-js

- Put the package into scope as @iohk-jormungandr/wallet-js
- Change the output file prefix to generate files named `wallet.js` etc.

## [0.5.0-pre5] - 2020-10-30

### Added

#### wallet-js

- Add Ed25519Extended generation from seed
- Key signing and verification.
- Add the other kinds of private keys: 
  - Ed25519

#### Cordova-android | Java | C | Electron/Browser
- vote_proposal_new_public
- vote_proposal_new_private

#### Cordova-electron/browser
- add confirm/get pending transactions
- add import keys (free keys)
- pin decryption (symmetric cipher decrypt)

### Deprecated

- proposal_new: In favour of the specific functions for each case. This
function takes an enum, which currently only can be used to cast public
votes (the internal function still uses rust enums, this is only for non-rust
apis).

## [0.5.0-pre4] - 2020-10-13

### Added

#### wallet-js

- Key pair generation support.
- Symmetric encryption and decryption support.

## [0.5.0-pre3] - 2020-09-04

### Fixed

- Decryption function now returns an error if the authentication fails.

### Added

#### wallet-cordova

- iOS support for the import key and decryption functions.

## [0.5.0-pre2] - 2020-08-18

### Fixed
- Wrong secret key type was used when recovering from mnemonics.

## [0.5.0-pre1] - 2020-08-18
### Added

- New utxo store.
- Allow recovering from single free utxo keys.
- Custom symmetric encryption/decryption module.

## [0.4.0] - 2020-07-08

## [0.4.0-pre3] - 2020-06-22

## [0.4.0-pre2] - 2020-06-04

## [0.4.0-pre1] - 2020-06-03

## [0.3.1] - 2020-22-05

## [0.3.0] - 2020-05-01

## [0.2.0] - 2020-04-15

## [0.1.0] - 2020-04-10
