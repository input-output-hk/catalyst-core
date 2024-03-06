---
title: Wallet Cryptography and Encoding
subtitle: 'Status: In Progress'
author: Vincent Hanquez, Nicolas Di Prima
documentclass: scrartcl
toc: t
numbersections: true
---

# Wallet Cryptography and Encoding

## Mnemonics encoding

We define a way for easily enter and write down arbitrary binary seed using a
simple dictionary of known words (available in many different languages).

The motivation here is to have sentence of words easy to read and write sentences,
which map uniquely back and forth to a sized binary data. It apply to any
binary, but usually is useful for binary data that need to be saved
or transfer, and need to be less sensible to human errors.

### Bip39

We used BIP39 dictionaries, and the BIP39 mnemonic encoding to represent
binaries of size multiple of 32 bits.

Valid BIP39 Encoding defined (where CS is Checksum Size):

| Num Mnemonic | Entropy & CS | Entropy Size        | CS     |
| ------------ | ------------ | ------------------- | ------ |
| 9 words      | 99 bits      | 96 bits (12 bytes)  | 3 bits |
| 12 words     | 132 bits     | 128 bits (16 bytes) | 4 bits |
| 15 words     | 165 bits     | 160 bits (20 bytes) | 5 bits |
| 18 words     | 198 bits     | 192 bits (24 bytes) | 6 bits |
| 21 words     | 231 bits     | 224 bits (28 bytes) | 7 bits |
| 24 words     | 264 bits     | 256 bits (32 bytes) | 8 bits |

Words of the dictionary are different enough that it should
be hard to swap by mistake one word by another close by,
There's also a lightweight checksuming mechanism which
prevent to some extend swapping some words of the dictionary
by another with reasonable probability.

## Origin Key generation

The origin key is the material that is used to generate everything else in a
given wallet. It should be composed of high quality randomness (also known as
entropy).

A valid seed is a binary value of 128 to 256 bits, in 32 bits increment (16 to
32 bytes in 4 bytes increment). Any other seed size is invalid and should be
rejected.

The size of the seed is only affecting the initial master key generation, after
that it doesn't have any effect (apart from the underlying security
implication).

Generation:

1. Generate 16, 20, 24, 28, 32 bytes (128 bits to 256 bits in 32 bits increment) as seed, from high quality random.
2. UX:

* Encode the bytes to mnemonics either in BIP39 and EME, or both.
* Display to the user for safe keeping (recovery, etc).

Recovery:

1. UX: Use the BIP39 dictionary for allowed words & autocompletion
2. UX: Detect whether it's a BIP39 (12,15,18,21,24 words) or EME encoding (13, 16, 19, 22, 25 words)
3. UX: Decode the words to binary

```
TODO test vector origin key to EME and BIP39
```

## Master key generation (to cryptographic key)

Inputs:

* Origin Key: A valid Origin Key as described above
* Password: any byte array of any size.

Output:

* A valid extended private key (XPrv)

A XPrv is 96 bytes, and composed of:

* 64 bytes: an extended ED25519 secret key composed of:
  * 32 bytes: ed25519 curve scalar, which should have some specific bits clear and some set (see tweakBits)
  * 32 bytes: ed25519 binary blob used as IV for signing
* 32 bytes: chain code for the ed25519-bip32 scheme

Generally we're narrowing the choice of primitives to Key Derivation Function
(KDF) to provide the binary extension from seed (12 to 24 bytes) to a larger
binary value (here 96 bytes).

(Example of existing KDF: pbkdf2, bcrypt, scrypt, argon2)

This system is a multi factor cryptographic retrieval, which is similar to
multi factor authentication (or more specifically 2FA) but applied to
cryptographic key retrieval:

* The seed: something we have: either newly generated or from a recovery
* The password: something we know: user chosen password

Given the lack of any structure requirement of the output bits, along with
providing a strong multi factor cryptographic material retrieval, this also
provide plausible deniability; It's not possible to identify whether the
password is correct (and correctly entered), short of looking for specific
usage recorded elsewhere (e.g. looks for own utxos on a wallet). This present
some challenge at the user level, as losing the password or not entering
correctly, lead to a brand new wallet. It is also valid to either leave the
password empty or hardcode it to a specific string, although that could create
wallet interoperability issues.

Along with those requirement, it should be (reasonably) slow to compute, to
prevent simple case of brute forcing in case of leak of the seed.

Given the password, we need to use a password based key derivation function.
Without any other external constraint, we would choose argon2, but given that
we need to be conservative for support reason and the ability to be embedded in
small hardware, we will choose pbkdf2 which enjoy wide support and large number
of implementations. Furthermore, it is also the primitive of choice for BIP39
master key generation.

Note that this choice doesn't protect against brute forcing from ASIC and GPU
implementations, however because of plausible deniability, even though the
function can be computed really fast on specialized hardware, it's still hard
to test whether or not the master key is the right one and thus terminate the
search. This final search step involves many derivations (BIP44), and many
address generation searches in an existing blockchain.

The algorithm for master key generation:

    masterKeyGeneration(seed, password) {
        data := PBKDF2(kdf=HMAC-SHA512, iter=4096,
                       salt=seed,
         password=password,
         outputLen=96);
        masterKey := tweakBits(data);
    }

    tweakBits(data) {
        // on the ed25519 scalar leftmost 32 bytes:
        // * clear the lowest 3 bits
        // * clear the highest bit
        // * clear the 3rd highest bit
        // * set the highest 2nd bit
        data[0] &= ~0b111;
        data[31] &= 0b00011111;
        data[31] |= 0b01000000;
    }

```
TODO add test vectors of seed+password to output
```

## Wallet Hierarchical Deterministic (HD) Key Derivation

Derivation is the process to transform a XPrv into another XPrv having a one way process:

    Parent XPrv ---> Child XPrv
    Child XPrv  -/-> Parent Xprv

We supplement this derivation with a hierarchy and 32 bits indices, and effectively turn
derivation into the ability to derive an "infinite"" number of keys, organised in a tree fashion,
with at the root of this binary, one unique XPrv, which by call the `master key`.

### Scheme

We use the scheme defined here, based on [Hierarchical Deterministic Keys over a Non-linear keyspace](https://cardanolaunch.com/assets/Ed25519_BIP.pdf)

## BIP44

we use:

* `H(X)` to indicate a hardened index X of `0 <= X <= 2^31-1` which encode a value of `2^31 + X`
* `S(X)` to indicate a softened index X of `0 <= X <= 2^31-1` which encode a value of `X`

We define the following derivation path, where we repeadtly derive 5 levels
from the root key to obtain a leaf:

    root / H(44) / H(COIN_TYPE) / H(ACCOUNT) / S(CHANGE) / S(INDEX)

### Account

This level splits the key space into independent user identities, so the wallet
never mixes the coins across different accounts.

Accounts are numbered from index 0 in sequentially increasing manner.

Software should prevent a creation of an account if a previous account does not
have a transaction history (meaning none of its addresses have been used
before).

### Change

CHANGE is 0 (used for external chain) or 1 (used for change address).

External chain is used for addresses that are meant to be visible outside of
the wallet (e.g. for receiving payments). Internal chain is used for addresses
which are not meant to be visible outside of the wallet and is used for return
transaction change.

### Extension to Accounting style

BIP44 is defined related to utxo, but we add another change constant of 2
to generate reusable accounts for a given bip44 account.

    root/H(44) / H(COIN_TYPE) / H(ACCOUNT) / S(2) / S(ACCOUNT-INDEX)
