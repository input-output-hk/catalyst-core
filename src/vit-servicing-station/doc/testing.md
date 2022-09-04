# vit-servicing-station-tests

Vit servicing station tests project is a container project vit-servicing-station tests. Tests are validating server correctness, stability and interaction with database/rest api. Also there are non-functional tests which verify node durability and reliability

## Quick start

### Prerequisites

In order to run test vit-servicing-station-server need to be installed or prebuilt.

### Start tests

In order to build vit-servicing-station in main project folder run:
```
cd vit-servicing-station-tests
cargo test
```

## Tests categories

Test are categories based on application/layer and property under test (functional or non-functional: load, perf etc.)

### How to run all functional tests

```
cd vit-servicing-station-tests
cargo test 
```

### How to run performance tests
```
cd vit-servicing-station-tests
cargo test --features non-functional
```

### How to run endurance tests
```
cd vit-servicing-station-tests
cargo test --features soak,non-functional
```

### Frequency
Functional tests are run on each PR. Performance and testnet integration tests are run nightly
