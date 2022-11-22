# integration-tests

Integration test is a container project for all catalyst e2e and integration tests. Tests are validating network correctness, stability. Also there are non-functional tests which verify node durability and reliability

## Architecture of tests

Integration tests architecture relies on test pyramid approach, where most of the effort is put into component and integration level and finally small amount of tests on E2E. Thanks to that we can create fast and reliable tests.

![Testing architecture](./graphs/testing-architecture.svg)

Before approaching Jormungandr testing we need to first remind ourselves a simplified architecture diagram for jcli & jormungandr.

![Simplified architecture](./graphs/catalyst-simplified-arch.svg)

## Quick start

### Prerequisites

In order to run test integration tests below components need to be installed or prebuild:
- [vit-servicing-station-server|https://github.com/input-output-hk/vit-servicing-station/tree/master/vit-servicing-station-server]
- [jormungandr|https://github.com/input-output-hk/jormungandr/tree/master/jormungandr]
- [valgrind|https://github.com/input-output-hk/vit-testing/tree/master/valgrdin]

### Start tests

In order to build jormungandr-automation in main project folder run:
```
cd testing
cargo test
```

## Tests categories

Test are categories based on application/layer and property under test (functional or non-functional: load, perf etc.)
Below diagram is a good overview:

![Test categories](./graphs/jormungandr-test-categories.svg)

### How to run all functional tests

```
cd integration-tests
cargo test
```

### How to run testnet functional tests
```
cd integration-tests
cargo test --features testnet-tests
```

### How to run load tests
```
cd integration-tests
cargo test non_functional::load --features load-tests
```

### How to run network endurance tests
```
cd testing/jormungandr-integration-tests
cargo test non_functional::soak  --features soak-tests
```

### Frequency
Functional tests are run on each PR. Performance and testnet integration tests are run nightly
