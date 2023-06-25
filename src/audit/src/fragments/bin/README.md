### Fragment analysis tool

This tool allows facilitates fragment analysis of a fund

*Example usage:*

```
cargo build --release -p audit
```  

#### Validate tally results vs published active plans

```bash

HISTORICAL_FUND_STORAGE='/tmp/fund9-leader-1/persist/leader-1'

./target/release/fragments --tally-fragments $HISTORICAL_FUND_STORAGE

```

- [x] Tick vs active plans