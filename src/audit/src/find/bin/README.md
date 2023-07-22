### Find my vote

Retrieve vote metadata given caster pub key      

*Example usage:*

```
cargo build --release -p audit
```  


```bash

CASTER_PUB_KEY='ca1qh9mq93uwkvre76p0zj3ya66tsmrcgevn28rrsjfp65vqp59yjljywkerav'
FRAGMENTS_STORAGE=/tmp/fund9-leader-1/persist/leader-1

./target/release/find --fragments $FRAGMENTS_STORAGE --pub-key $CASTER_PUB_KEY

```

