# valgrind

Valgrind is a Rest API project which is simplified proxy solution for catalyst backend.

## build

In main project folder run:

```sh
cd valgrind
cargo build
```

and install:

`cargo install --path .`

## quick start

The simplest configuration is available by using command:

`valgrind --block0_path block0.bin`

By default valgrind will be exposed at `127.0.0.1:8000`

## client

Valgrind project provides also API for interacting with proxy server. Usage example:

```rust
    use valgrind::client::{ValgrindClient,ValgrindSettings};


    let settings = RestSettings {
        enable_debug: false,
        use_https: false,
        certificate: None,
        cors: None,
    }

    let address = "0.0.0.0:8080".to_string();

    let client = ValgrindClient::new(address, settings)
    let fragment_logs = client.fragment_logs()?;

```
