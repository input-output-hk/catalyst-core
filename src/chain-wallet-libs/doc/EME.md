### Enhanced Mnemonic Encoding (EME) -- Draft

We add an extra mnemonic word to the encoding, this allows us to have an extra
11 bits for the scheme metadata. The EME scheme:

* Improves the detection and versioning compared to the standard BIP39
* EME mnemonics is not accepted by a compliant BIP39 implementation by always having 1 extra word
* EME mnemonics maps trivially to a BIP39 mnemonics by stripping the first word.

The first 2 bits are defined for the type, which given different type imply
different parsing of the following bits

* 2 bits : where the value represent:
  * 0: wallet key seed
  * 1: reserved for paper wallet
  * 2: reserved for extension. UX should report unknown type
  * 3: reversed for extension. UX should report unknown type

The following apply to wallet key seed:

* 2 bits : master key derivation version.
  * Version 0: use the scheme defined below
* 3 bits : mnemonic size (invalid=0b000,12=0b001, 15=0b010, 18=0b011, 21=0b100, 24=0b101, and by extension: 27=0b110, 30=0b111) 
* 4 bits : checksum of the first 2 words (22bits) 0b1111. How is it computed ? 

The mnemonic size allows after the words to determine the number of words are
expected for this instance. This can be used for the UI either during the
process of filling the mnemonic or at the end to do simple validation.

The 2 words checksum allows for early detection of the validity of the EME.
In the UX, on the 3rd word entered, early checksuming can allow detection, with
some margin of error, whether or not the EME encoding is valid. This allow for
extra feedback during the UX, and help detect classic bip39 scheme from EME
scheme.

```
TODO test vectors
```

