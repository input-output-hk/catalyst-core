### Find my vote

Retrieve voting key history and metadata via offline fragment analysis replay.    

*Example usage:*

```
cargo build --release -p audit
```  

Voting key is present in the first field as per defined in [CIP-36](https://cips.cardano.org/cips/cip36/); user is given a link to cardanoscan after they vote, their voting key is present in the [metadata, e.g](https://cardanoscan.io/transaction/9f3706e8b26bc0c88ef74e0b020bf148dc90301e3a1e3c465db1f4d206729b7b?tab=metadata)


```bash

VOTING_KEY='e5b0a5c250f78b574b8b17283bcc6c7692f72fc58090f4a0a2362497d28d1a85'
FRAGMENTS_STORAGE=/tmp/fund9-leader-1/persist/leader-1

./target/release/find --fragments $FRAGMENTS_STORAGE --voting-key $VOTING_KEY

```

### Aggregrate all voter keys and write to file
```bash

FRAGMENTS_STORAGE=/tmp/fund9-leader-1/persist/leader-1
./target/release/find --fragments $FRAGMENTS_STORAGE --aggregate true

```

### Convert key formats
```bash

VOTING_KEY='e5b0a5c250f78b574b8b17283bcc6c7692f72fc58090f4a0a2362497d28d1a85'

./target/release/find --key-to-convert $VOTING_KEY


VOTING_KEY='ca1q0uftf4873xazhmhqrrqg4kfx7fmzfqlm5w80wake5lu3fxjfjxpk6wv3f7'

./target/release/find --key-to-convert $VOTING_KEY

```

### Check a batch of keys presented in a file format and write key metadata to a file.
```bash

KEY_FILE='/tmp/keyfile.txt'
FRAGMENTS_STORAGE=/tmp/fund9-leader-1/persist/leader-1

./target/release/find --fragments $FRAGMENTS_STORAGE --key-file $KEY_FILE

```

