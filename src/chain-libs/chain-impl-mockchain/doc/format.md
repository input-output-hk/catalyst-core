**This is a draft document**

# Preliminaries

All integers are encoded in big-endian format.

`Signature` has the format

    Length | Payload

where `Length` is a 16-bit unsigned integer `N`, and `Payload` is `N`
bytes of signature data.

# Block

Format is:

    Header | Content

## Block Header

The header is a small piece of data, containing enough informations for validation and network deduplication and a strong signed cryptographic link to the content.

Common (2 * 64 bits + 1 * 32 bits + 2 * 256 bits = 84 bytes):

* Size of Header: 2 bytes (16 bits): Maximum header is thus 64K not including the block content
* Version of block: 2 bytes (16 bits)
* Size of Content: 4 bytes (32 bits)
* Block Date: Epoch (4 bytes, 32 bits) + Slot-id (4 bytes - 32 bits)
* Chain length (number of ancestor blocks; first block has chain length 0): 4 bytes (32 bits)
* Hash of content `H(Content)` (32 bytes - 256 bits)
* Parent Header hash : 32 bytes (256 bits)

We reserved the special value of all 0 for the parent header hash, to
represent the lack of parent for the block0, but for other blocks it's not
reserved and could represent, although with negligeable probability, a valid
block. In any case, it means that there's no special meaning to this value in
normal context.

In BFT the header also contains (768 bits = 96 bytes):

* BFT Public Key of the leader (32 bytes)
* BFT Signature (64 bytes)

In Praos/Genesis the header also contains (612 bytes):

* VRF PubKey: 32 bytes (ristretto25519)
* VRF Proof: 96 bytes (ristretto25519 DLEQs)
* KES Signature: 484 bytes (sumed25519-12)

Additionally, we introduce the capability to address each header individually
by using a cryptographic hash function : `H(HEADER)`. The hash include all
the content serialized in the sequence above, except the size of header,
which effectively means that calculating the hash of a fully serialized
header is just applying the hash function to the binary data except the first
2 bytes.

## Block Body

We need to be able to have different type of content on the blockchain, we also
need a flexible system for future expansion of this content.  The block content
is effectively a sequence of serialized content, one after another.

Each individual piece of block content is called a fragment and is prefixed
with a header which contains the following information:

* Size of content piece in bytes (2 bytes)
* Type of piece (1 byte): up to 256 different type of block content.

The block body is formed of the following stream of data:

    HEADER(FRAGMENT1) | FRAGMENT1 | HEADER(FRAGMENT2) | FRAGMENT2 ...

Where HEADER is:

	SIZE (2 bytes) | TYPE (1 byte) | 00 (1 byte)

Additionally, we introduce the capability to refer to each fragment
individually by FragmentId, using a cryptographic hash function :

    FragmentId = H(TYPE | FRAGMENT-CONTENT)

The hash doesn't include the size prefix in the header to simplify
calculation of hash with on-the-fly (non serialized) structure.

Types of content:

* Transaction
* Old Transaction
* Owner stake Delegation
* Certificate (Staking, Pool, Delegation, ...)
* TBD Update
* TBD Debug Stats : block debug information in chain.

### Common Structure

Fragment contents unless otherwise specify are in the following generic format:

    1. PAYLOAD
    2. INPUTS/OUTPUTS
    3. WITNESSNES(using 1+2 as message)
    4. PAYLOAD-AUTHENTICATION(using 1+2+3 as message)

PAYLOAD can be empty depending on the specific message. PAYLOAD-AUTHENTICATION allows
binding the PAYLOAD with the Witness to prevent replayability when necessary, and
its actual content is linked to the PAYLOAD and can be empty too.

This construction is generic and allow payments to occurs for either transfer of value
and/or fees payment, whilst preventing replays.

#### Inputs/Outputs

Inputs/Outputs is in the following format:

    IOs = #INPUTS (1 byte) | #OUTPUTS (1 byte) | INPUT1 | .. | OUTPUT1 | ..

* Input number : 1 byte: 256 inputs maximum
* Output number : 1 byte where 0xff is reserved: 255 outputs maximum
* Transaction Inputs (Input number of time * 41 bytes):
  * Index (1 byte) : special value 0xff specify a account spending (single or multi)
  * Account Identifier or Utxo Identifier (also FragmentId) (32 bytes)
  * Value (8 bytes)
* Transaction Outputs (Output number of time):
  * Address (bootstrap address 33 bytes, delegation address 65 bytes, account address 33 bytes)
  * Value (8 bytes)

#### Witnesses

To authenticate the PAYLOAD and the IOs, we add witnesses with a 1-to-1 mapping
with inputs. The serialized sequence of inputs, is directly linked with the
serialized sequence of witnesses.

Fundamentally the witness is about signing a message and generating/revealing
cryptographic material to approve unequivocally the content.

There's currently 3 differents types of witness supported:

* Old utxo scheme: an extended public key, followed by a ED25519 signature
* utxo scheme: a ED25519 signature
* Account scheme: a counter and an ED25519 signature

With the following serialization:

* Type of witness: 1 byte
* Then either:
  * Type=1 Old utxo witness scheme (128 bytes):
    * ED25519-Extended Public key (64 bytes)
    * ED25519 Signature (64 bytes)
  * Type=2 utxo witness scheme (64 bytes):
    * ED25519 Signature (64 bytes)
  * Type=3 Account witness (68 bytes):
    * Account Counter (4 bytes : TODO-ENDIANNESS)
    * ED25519 Signature (64 bytes)

The message, w.r.t the cryptographic signature, is generally of the form:

    TRANSACTION-SIGN-DATA-HASH = H(PAYLOAD | IOs)
    Authenticated-Data = H(HEADER-GENESIS) | TRANSACTION-SIGN-DATA-HASH | WITNESS-SPECIFIC-DATA

#### Rationale

* 1 byte index utxos: 256 utxos = 10496 bytes just for inputs, already quite big and above a potential 8K soft limit for block content
Utxo representation optimisations (e.g. fixed sized bitmap)

* Values in inputs:
Support for account spending: specifying exactly how much to spend from an account.
Light client don't have to trust the utxo information from a source (which can lead to e.g. spending more in fees), since a client will now sign a specific known value.

* Account Counter encoding:
4 bytes: 2^32 unique spending from the same account is not really reachable:
10 spending per second = 13 years to reach limit.
2^32 signatures on the same signature key is stretching the limits of scheme.
Just the publickey+witnesses for the maximum amount of spending would take 400 gigabytes

* Value are encoded as fixed size integer of 8 bytes (TODO: specify endianness),
instead of using any sort of VLE (Variable Length Encoding). While it does
waste space for small values, it does this at the net advantages of
simplifying handling from low memory devices by not having need for a
specific serialization format encoder/decoder and allowing value changing in
binary format without having to reduce or grow the binary representation.
This

## Type 0: Initial blockchain configuration

This message type may only appear in the genesis block (block 0) and
specifies various configuration parameters of the blockchain. Some of
these are immutable, while other may be changed via the update
mechanism (see below). The format of this message is:

    ConfigParams

where `ConfigParams` consists of a 16-bit field denoting the number of
parameters, followed by those parameters:

    Length | ConfigParam*{Length}

`ConfigParam` has the format:

    TagLen Payload

where `TagLen` is a 16-bit bitfield that has the size of the payload
(i.e. the value of the parameter) in bytes in the 6 least-significant
bits, and the type of the parameter in the 12 most-significant
bits. Note that this means that the payload cannot be longer than 63
bytes.

The following parameter types exist:

| tag  | name                                 | value type | description                                                                            |
| :--- | :----------------------------------- | :--------- | :------------------------------------------------------------------------------------- |
| 1    | discrimination                       | u8         | address discrimination; 1 for production, 2 for testing                                |
| 2    | block0-date                          | u64        | the official start time of the blockchain, in seconds since the Unix epoch             |
| 3    | consensus                            | u16        | consensus version; 1 for BFT, 2 for Genesis Praos                                      |
| 4    | slots-per-epoch                      | u32        | number of slots in an epoch                                                            |
| 5    | slot-duration                        | u8         | slot duration in seconds                                                               |
| 6    | epoch-stability-depth                | u32        | the length of the suffix of the chain (in blocks) considered unstable                  |
| 8    | genesis-praos-param-f                | Milli      | determines maximum probability of a stakeholder being elected as leader in a slot      |
| 9    | max-number-of-transactions-per-block | u32        | maximum number of transactions in a block                                              |
| 10   | bft-slots-ratio                      | Milli      | fraction of blocks to be created by BFT leaders                                        |
| 11   | add-bft-leader                       | LeaderId   | add a BFT leader                                                                       |
| 12   | remove-bft-leader                    | LeaderId   | remove a BFT leader                                                                    |
| 13   | allow-account-creation               | bool (u8)  | 0 to enable account creation, 1 to disable                                             |
| 14   | linear-fee                           | LinearFee  | coefficients for fee calculations                                                      |
| 15   | proposal-expiration                  | u32        | number of epochs until an update proposal expires                                      |
| 16   | kes-update-speed                     | u32        | maximum number of seconds per update for KES keys known by the system after start time |

`Milli` is a 64-bit entity that encoded a non-negative, fixed-point
number with a scaling factor of 1000. That is, the number 1.234 is
represented as the 64-bit unsigned integer 1234.

`LinearFee` has the format:

    Constant | Coefficient | Certificate

all of them 64-bit unsigned integers, specifying how fees are computed
using the formula:

    Constant + Coefficient * (inputs + outputs) + Certificate * certificates

where `inputs`, `outputs` and `certificates` represent the size of the
serialization of the corresponding parts of a transaction in bytes.

## Type 2: Transaction

Transaction is the composition of the TokenTransfer structure followed directly by the witnesses. PAYLOAD needs to be empty. Effectively:

    TokenTransfer<PAYLOAD = ()> | Witnesses

TODO:

* Multisig
* Fees

## Type 2: OwnerStakeDelegation

    TokenTransfer<PAYLOAD = OwnerStakeDelegation> | Witnesses

    OwnerStakeDelegation = DelegationType

## Type 3: Certificate

Certificate is the composition of the TokenTransfer structure where PAYLOAD is the certificate data, and then the witnesses. Effectively:

    TokenTransfer<PAYLOAD = CERTIFICATE> | Witnesses

Known Certificate types:

* Staking declaration: declare a staking key + account public information
* Stake pool registration: declare the VRF/KES key for a node.
* Delegation: contains a link from staking to stake pool.

Content:

* PublicKey
* Signature of the witness with the private key associated to the revealed PublicKey

## Type 4: Update Proposal

Update proposal messages propose new values for blockchain
settings. These can subsequently be voted on. They have the following
form:

    Proposal | ProposerId | Signature

where `ProposerId` is a ed25519 extended public key, and `Signature`
is a signature by the corresponding private key over the string
`Proposal | ProposerId`.

`Proposal` has the following format:

    ConfigParams

where `ConfigParams` is defined above.

## Type 5: Update votes

Vote messages register a positive vote for an earlier update
proposal. They have the format

    ProposalId | VoterId | Signature

where `ProposalId` is the message ID of an earlier update proposal
message, `VoterId` is an ed25519 extended public key, and `Signature`
is a signature by the corresponding secret key over `ProposalId |
VoterId`.

## Type 11: Vote Cast

VoteCast message is used to vote for a particular voting event.

VoteCast transaction should have only 1 input, 0 output and 1 witness (signature).

Full fragment representation in hex:
```
0000037e000b36ad42885189a0ac3438cdb57bc8ac7f6542e05a59d1f2e4d1d38194c9d4ac7b000203f6639bdbc9235103825a9f025eae5cff3bd9c9dcc0f5a4b286909744746c8b6fb0018773d3b4308344d2e90599cd03749658561787eab714b542a5ccaf078846f6639bdbc9235103825a9f025eae5cff3bd9c9dcc0f5a4b286909744746c8b6fc8f58976fc0e951ba284a24f3fc190d914ae53aebcc523e7a4a330c8655b4908f6639bdbc9235103825a9f025eae5cff3bd9c9dcc0f5a4b286909744746c8b6fb0018773d3b4308344d2e90599cd03749658561787eab714b542a5ccaf078846021c76d0a50054ef7205cb95c1fd3f928f224fab8a8d70feaf4f5db90630c3845a06df2f11c881e396318bd8f9e9f135c2477e923c3decfd6be5466d6166fb3c702edd0d1d0a201fb8c51a91d01328da257971ca78cc566d4b518cb2cd261f96644067a7359a745fe239db8e73059883aece4d506be71c1262b137e295ce5f8a0aac22c1d8d343e5c8b5be652573b85cba8f4dcb46cfa4aafd8d59974e2eb65f480cf85ab522e23203c4f2faa9f95ebc0cd75b04f04fef5d4001d349d1307bb5570af4a91d8af4a489297a3f5255c1e12948787271275c50386ab2ef3980d882228e5f3c82d386e6a4ccf7663df5f6bbd9cbbadd6b2fea2668a8bf5603be29546152902a35fc44aae80d9dcd85fad6cde5b47a6bdc6257c5937f8de877d5ca0356ee9f12a061e03b99ab9dfea56295485cb5ce38cd37f56c396949f58b0627f455d26e4c5ff0bc61ab0ff05ffa07880d0e5c540bc45b527e8e85bb1da469935e0d3ada75d7d41d785d67d1d0732d7d6cbb12b23bfc21dfb4bbe3d933eaa1e5190a85d6e028706ab18d262375dd22a7c1a0e7efa11851ea29b4c92739aaabfee40353453ece16bda2f4a2c2f86e6b37f6de92dc45dba2eb811413c4af2c89f5fc0859718d7cd9888cd8d813da2e93726484ea5ce5be8ecf1e1490b874bd897ccd0cbc33db0a1751f813683724b7f5cf750f2497953607d1e82fb5d1429cbfd7a40ccbdba04fb648203c91e0809e497e80e9fad7895b844ba6da6ac690c7ce49c10e00000000000000000100ff00000000000000036d2ac8ddbf6eaac95401f91baca7f068e3c237386d7c9a271f5187ed909155870200000000e6c8aa48925e37fdab75db13aca7c4f39068e12eeb3af8fd1f342005cae5ab9a1ef5344fab2374e9436a67f57041899693d333610dfe785d329988736797950d
```
1. Fragment size (u32): `0000037e`
2. `00`
3. Fragment id tag (u8): `0b` == `11` (it is equal to VoteCast tag)
4. Vote plan id (32 byte hash): `36ad42885189a0ac3438cdb57bc8ac7f6542e05a59d1f2e4d1d38194c9d4ac7b`
5. Proposal index (u8): `00`
6. Payload type tag (u8): `02`
7. Encrypted vote: 
`03|f6639bdbc9235103825a9f025eae5cff3bd9c9dcc0f5a4b286909744746c8b6f|b0018773d3b4308344d2e90599cd03749658561787eab714b542a5ccaf078846|f6639bdbc9235103825a9f025eae5cff3bd9c9dcc0f5a4b286909744746c8b6f|c8f58976fc0e951ba284a24f3fc190d914ae53aebcc523e7a4a330c8655b4908|f6639bdbc9235103825a9f025eae5cff3bd9c9dcc0f5a4b286909744746c8b6f|b0018773d3b4308344d2e90599cd03749658561787eab714b542a5ccaf078846`
    - size (u8): `03` 
    - ciphertext (group element (32 byte), group element (32 byte)): `f6639bdbc9235103825a9f025eae5cff3bd9c9dcc0f5a4b286909744746c8b6f|b0018773d3b4308344d2e90599cd03749658561787eab714b542a5ccaf078846|f6639bdbc9235103825a9f025eae5cff3bd9c9dcc0f5a4b286909744746c8b6f|c8f58976fc0e951ba284a24f3fc190d914ae53aebcc523e7a4a330c8655b4908|f6639bdbc9235103825a9f025eae5cff3bd9c9dcc0f5a4b286909744746c8b6f|b0018773d3b4308344d2e90599cd03749658561787eab714b542a5ccaf078846`
8. Proof: `02|1c76d0a50054ef7205cb95c1fd3f928f224fab8a8d70feaf4f5db90630c3845a|06df2f11c881e396318bd8f9e9f135c2477e923c3decfd6be5466d6166fb3c70|2edd0d1d0a201fb8c51a91d01328da257971ca78cc566d4b518cb2cd261f9664|4067a7359a745fe239db8e73059883aece4d506be71c1262b137e295ce5f8a0a|ac22c1d8d343e5c8b5be652573b85cba8f4dcb46cfa4aafd8d59974e2eb65f48|0cf85ab522e23203c4f2faa9f95ebc0cd75b04f04fef5d4001d349d1307bb557|0af4a91d8af4a489297a3f5255c1e12948787271275c50386ab2ef3980d88222|8e5f3c82d386e6a4ccf7663df5f6bbd9cbbadd6b2fea2668a8bf5603be295461|52902a35fc44aae80d9dcd85fad6cde5b47a6bdc6257c5937f8de877d5ca0356|ee9f12a061e03b99ab9dfea56295485cb5ce38cd37f56c396949f58b0627f455|d26e4c5ff0bc61ab0ff05ffa07880d0e5c540bc45b527e8e85bb1da469935e0d|3ada75d7d41d785d67d1d0732d7d6cbb12b23bfc21dfb4bbe3d933eaa1e5190a|85d6e028706ab18d262375dd22a7c1a0e7efa11851ea29b4c92739aaabfee403|53453ece16bda2f4a2c2f86e6b37f6de92dc45dba2eb811413c4af2c89f5fc08|59718d7cd9888cd8d813da2e93726484ea5ce5be8ecf1e1490b874bd897ccd0c|bc33db0a1751f813683724b7f5cf750f2497953607d1e82fb5d1429cbfd7a40c|cbdba04fb648203c91e0809e497e80e9fad7895b844ba6da6ac690c7ce49c10e`
    - size (u8): `02`
    - announcements (group element (32 byte), group element (32 byte), group element (32 byte)): `1c76d0a50054ef7205cb95c1fd3f928f224fab8a8d70feaf4f5db90630c3845a|06df2f11c881e396318bd8f9e9f135c2477e923c3decfd6be5466d6166fb3c70|2edd0d1d0a201fb8c51a91d01328da257971ca78cc566d4b518cb2cd261f9664|4067a7359a745fe239db8e73059883aece4d506be71c1262b137e295ce5f8a0a|ac22c1d8d343e5c8b5be652573b85cba8f4dcb46cfa4aafd8d59974e2eb65f48|0cf85ab522e23203c4f2faa9f95ebc0cd75b04f04fef5d4001d349d1307bb557`
    - ciphertext (group element (32 byte), group element (32 byte)): `0af4a91d8af4a489297a3f5255c1e12948787271275c50386ab2ef3980d88222|8e5f3c82d386e6a4ccf7663df5f6bbd9cbbadd6b2fea2668a8bf5603be295461|52902a35fc44aae80d9dcd85fad6cde5b47a6bdc6257c5937f8de877d5ca0356|ee9f12a061e03b99ab9dfea56295485cb5ce38cd37f56c396949f58b0627f455`
    - response randomness (scalar (32 byte), scalar (32 byte), scalar (32 byte)): `d26e4c5ff0bc61ab0ff05ffa07880d0e5c540bc45b527e8e85bb1da469935e0d|3ada75d7d41d785d67d1d0732d7d6cbb12b23bfc21dfb4bbe3d933eaa1e5190a|85d6e028706ab18d262375dd22a7c1a0e7efa11851ea29b4c92739aaabfee403|53453ece16bda2f4a2c2f86e6b37f6de92dc45dba2eb811413c4af2c89f5fc08|59718d7cd9888cd8d813da2e93726484ea5ce5be8ecf1e1490b874bd897ccd0c|bc33db0a1751f813683724b7f5cf750f2497953607d1e82fb5d1429cbfd7a40c`
    - scalar (32 byte): `cbdba04fb648203c91e0809e497e80e9fad7895b844ba6da6ac690c7ce49c10e`
9. IOW stand for Inputs-Outputs-Witnesses: `00000000000000000100ff00000000000000036d2ac8ddbf6eaac95401f91baca7f068e3c237386d7c9a271f5187ed909155870200000000e6c8aa48925e37fdab75db13aca7c4f39068e12eeb3af8fd1f342005cae5ab9a1ef5344fab2374e9436a67f57041899693d333610dfe785d329988736797950d`
    - block date (epoch (u32), slot (u32)): `00000000|00000000`
    - number of inputs and witnesses (u8): `01`
    - number of outputs (u8): `00`
    - Inputs
    1. 
        - index or accout (u8): `ff` (index)
        - value (u64): `0000000000000003`
        - input pointer (32 byte): `6d2ac8ddbf6eaac95401f91baca7f068e3c237386d7c9a271f5187ed90915587`
    - Witnesses
    1. 
        - witness type tag (u8): `02`
        - nonce (u32): `00000000`
        - legacy signature (64 byte): `e6c8aa48925e37fdab75db13aca7c4f39068e12eeb3af8fd1f342005cae5ab9a1ef5344fab2374e9436a67f57041899693d333610dfe785d329988736797950d`

Signing valid VoteCast fragment example (witness generation).


Transaction data to sign:
```
36ad42885189a0ac3438cdb57bc8ac7f6542e05a59d1f2e4d1d38194c9d4ac7b000203f6639bdbc9235103825a9f025eae5cff3bd9c9dcc0f5a4b286909744746c8b6fb0018773d3b4308344d2e90599cd03749658561787eab714b542a5ccaf078846f6639bdbc9235103825a9f025eae5cff3bd9c9dcc0f5a4b286909744746c8b6fc8f58976fc0e951ba284a24f3fc190d914ae53aebcc523e7a4a330c8655b4908f6639bdbc9235103825a9f025eae5cff3bd9c9dcc0f5a4b286909744746c8b6fb0018773d3b4308344d2e90599cd03749658561787eab714b542a5ccaf078846021c76d0a50054ef7205cb95c1fd3f928f224fab8a8d70feaf4f5db90630c3845a06df2f11c881e396318bd8f9e9f135c2477e923c3decfd6be5466d6166fb3c702edd0d1d0a201fb8c51a91d01328da257971ca78cc566d4b518cb2cd261f96644067a7359a745fe239db8e73059883aece4d506be71c1262b137e295ce5f8a0aac22c1d8d343e5c8b5be652573b85cba8f4dcb46cfa4aafd8d59974e2eb65f480cf85ab522e23203c4f2faa9f95ebc0cd75b04f04fef5d4001d349d1307bb5570af4a91d8af4a489297a3f5255c1e12948787271275c50386ab2ef3980d882228e5f3c82d386e6a4ccf7663df5f6bbd9cbbadd6b2fea2668a8bf5603be29546152902a35fc44aae80d9dcd85fad6cde5b47a6bdc6257c5937f8de877d5ca0356ee9f12a061e03b99ab9dfea56295485cb5ce38cd37f56c396949f58b0627f455d26e4c5ff0bc61ab0ff05ffa07880d0e5c540bc45b527e8e85bb1da469935e0d3ada75d7d41d785d67d1d0732d7d6cbb12b23bfc21dfb4bbe3d933eaa1e5190a85d6e028706ab18d262375dd22a7c1a0e7efa11851ea29b4c92739aaabfee40353453ece16bda2f4a2c2f86e6b37f6de92dc45dba2eb811413c4af2c89f5fc0859718d7cd9888cd8d813da2e93726484ea5ce5be8ecf1e1490b874bd897ccd0cbc33db0a1751f813683724b7f5cf750f2497953607d1e82fb5d1429cbfd7a40ccbdba04fb648203c91e0809e497e80e9fad7895b844ba6da6ac690c7ce49c10e00000000000000000100ff00000000000000036d2ac8ddbf6eaac95401f91baca7f068e3c237386d7c9a271f5187ed90915587
```
It consists of (detailed representation you can see below):
1. Vote plan id
2. Proposal index 
3. Payload type tag
4. Encrypted vote
5. Proof
6. Inputs

`blake2b256` hash of the transaction data to sign equals to `f51473df863be3e0383ce5a8da79c7ff51b3d98dadbbefbf9f042e8601901269`

Expected witness (includes signature)
```
0200000000e6c8aa48925e37fdab75db13aca7c4f39068e12eeb3af8fd1f342005cae5ab9a1ef5344fab2374e9436a67f57041899693d333610dfe785d329988736797950d
```

## Shared formats

Delegation Type has 3 different encodings:

```
No Delegation:

    00

Full delegation to 1 node:

    01 POOL-ID

Ratio delegation:

    Byte(PARTS) Byte(#POOLS) ( Byte(POOL-PARTS) POOL-ID )[#POOLS times]

    with PARTS >= 2 and #POOLS >= 2
```

Thus the encodings in hexadecimal:

```

No Delegation:

    00

Full Delegation to POOL-ID f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0

    01 f0 f0 f0 f0 f0 f0 f0  f0 f0 f0 f0 f0 f0 f0 f0
    f0 f0 f0 f0 f0 f0 f0 f0  f0 f0 f0 f0 f0 f0 f0 f0
    f0

Ratio Delegation of:
* 1/4 to POOL-ID f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0
* 3/4 to POOL-ID abababababababababababababababababababababababababababababababab

    04 02 01 f0 f0 f0 f0 f0  f0 f0 f0 f0 f0 f0 f0 f0
    f0 f0 f0 f0 f0 f0 f0 f0  f0 f0 f0 f0 f0 f0 f0 f0
    f0 f0 f0 03 ab ab ab ab  ab ab ab ab ab ab ab ab
    ab ab ab ab ab ab ab ab  ab ab ab ab ab ab ab ab
    ab ab ab ab
```
