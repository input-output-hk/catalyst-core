### Offline Fragment analysis tool

This tool allows facilitates fragment analysis of a fund using historical blockchain state

*Example usage:*

```
cargo build --release -p audit
```  

#### Cross reference official published results.

```bash

HISTORICAL_FUND_STORAGE='/tmp/fund9-leader-1/persist/leader-1'

OFFICIAL_CATALYST_RESULTS='/tmp/active_vote_plans_official.json'

./target/release/fragments --tally-fragments $HISTORICAL_FUND_STORAGE --active-vote-plans $OFFICIAL_CATALYST_RESULTS

```

#### Find my vote?
*TODO*