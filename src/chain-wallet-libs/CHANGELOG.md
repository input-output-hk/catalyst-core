# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

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
