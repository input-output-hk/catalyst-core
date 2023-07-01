### Offline Fragment analysis tool

This tool allows facilitates fragment analysis of a fund using historical blockchain state.       

*Example usage:*

```
cargo build --release -p audit
```  

#### Cross reference offline regenerated tally with official published results.

```bash

OFFICIAL_RESULTS=/tmp/activevoteplans.json 
BLOCK0=/tmp/fund9-leader-1/artifacts/block0.bin
FRAGMENTS_STORAGE=/tmp/fund9-leader-1/persist/leader-1

./target/release/offline --fragments $FRAGMENTS_STORAGE --block0 $BLOCK0 --official-results $OFFICIAL_RESULTS

```

#### Find my vote?
*TODO*