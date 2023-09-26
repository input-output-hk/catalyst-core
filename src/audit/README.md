# Audit Tooling:

## Offline audit

### Download Fund State
Download historical fund state from [*here*](https://github.com/input-output-hk/catalyst-core) in order to replay and audit the voting event.

The official published results can be found in this file in the form of **activevoteplans.json**.

**activevoteplans.json** = FINAL RESULTS.

If you would like to re-generate **activevoteplans.json** yourself, via a live node and historical fragments - [*see here for instructions*](./balance/README.md)

If not, you can begin the audit with the following steps.

*Example usage:*

```
cargo build --release -p audit
```  

*Cross reference offline tallies with published catalyst tallies.*

```bash

OFFICIAL_RESULTS=/tmp/activevoteplans.json 
BLOCK0=/tmp/fund9-leader-1/artifacts/block0.bin
FRAGMENTS_STORAGE=/tmp/fund9-leader-1/persist/leader-1

./target/release/offline --fragments $FRAGMENTS_STORAGE --block0 $BLOCK0 --official-results $OFFICIAL_RESULTS

```

This will create three files:
- *ledger_after_tally.json* **(decrypted ledger state after tally)** *should match official results!*
- *ledger_before_tally.json* **(encrypted ledger state before tally)** 
- *decryption_shares.json* **(decryption shares for each proposal)**

[*See here for next steps of audit process*](src/tally/README.md)

### Find my vote
[*See here for instructions on how to find your voting history*](src/find/README.md)