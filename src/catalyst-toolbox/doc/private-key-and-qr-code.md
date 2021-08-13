# Private Key Transfer Protocol

The encoding described in this document is used to export a private key
for use in another application.

In Catalyst, this format is used in the QR code used to pass the private key
from a wallet to the mobile application.

## Serialization format

We will expect the scheme to be updateable. So the first byte of the data
transferred refers to the protocol version in use. Here we reserve 0 for error.
1 for the protocol that is going to be defined in this document.

Once reading the first byte is 0b0000_0001 we can assume all the remaining bytes
are part of the scheme.

The first following 16 bytes are the salt.
Then the following 12 bytes are the nonce.
Then the encrypted data and then the last 16 bytes are for the tag.

```
+---------+----------+----------+----------------+----------+
| Version | Salt     | Nonce    | Encrypted Data | Tag      |
+---------+----------+----------+----------------+----------+
| 0x01    | 16 bytes | 12 bytes |                | 16 bytes |
+---------+----------+----------+----------------+----------+
```

The encrypted data unit is expected to be the binary representation of an
extended ed25519 key, 64 bytes in length.
In this application, we do not need the chain code as we will not do
any derivation.
So if 256 bytes of data are encoded, we are expecting 4 Ed25519 Extended keys.

# Symmetric encryption

Inputs:

* Password: a byte array
* Data: a byte array

Algorithm:

1. Generate a SALT of 16 bytes (only for encryption, on decryption the SALT is provided)
2. Generate a NONCE of 12 bytes (only for encryption, on decryption the NONCE is provided)
3. Derive the symmetric encryption key from the password:
   * Use PBKDF2 HMAC SHA512
   * 12983 iterations
   * Use the SALT
4. Encrypt the data (or decrypt)
   * Use ChaCha20Poly1305
   * Use the symmetric encryption key derived in step 3
   * Use the NONCE

Outputs:
* Encode the result in the format defined in the previous section
* Memzero everything else
