# Iapyx

Iapyx is a wallet cli & wallet api project which operates on catalyst-jormungandr network.

WARNING: main purpose of the wallet is testing. Do NOT use it on production.

## Build & Install

In order to build iapyx in main project folder run:

```
cd iapyx
cargo build
cargo install --path . --force
```

## Quick Start

### CLI

Iapyx can be used as a wallet cli. It is capable of sending votes or pull data from node. The simplest usage example is available by using commands:

* register new wallet based on qr code:
`iapyx wallets import qr qr_file.png --pin 1234`

* connect to node rest API:
`iapyx connect https://catalyst-backend.io/api`

* use recently created wallet for rest of commands:
`iapyx wallets use darek`

* sync with the node and get wallet data:
`iapyx wallets refresh`

* send vote:
`iapyx vote single --choice yes --pin --id {proposal_id}`

### API

Iapyx can be used as api in order to perform voting operations from the code:

```

    let wallet_proxy = spawn_network(...);
    let secret_file_path = Path::new("wallet_alice");
   

    let mut alice = iapyx::ControllerBuilder::default()
        .with_backend_from_client(wallet_proxy.client())?
        .with_wallet_from_secret_file(secret_file_path.as_ref())?
        .build()

    let proposals = alice.proposals().unwrap();
    let votes_data = proposals
        .iter()
        .take(batch_size)
        .map(|proposal| (proposal, Choice::new(0)))
        .collect();

    let fragment_ids = alice
        .votes_batch(votes_data)
        .unwrap()
        .iter()
        .map(|item| item.to_string())
        .collect();
```

## Configuration

Iapyx api doesn't use any configuration files. However cli uses small cache folder on filesystem (located in: `~/.iapyx`).
The purpose of this configuration is to store wallet lists as well as secret keys guarded by pass phrase.

### full list of available commands

Full list of commands is available on `iapyx --help` command.

```
iapyx 0.0.1
Command line wallet for testing Catalyst

USAGE:
    iapyx.exe <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    address                 Gets address of wallet in bech32 format
    clear-tx                Clears pending transactions to confirm. In case if expiration occurred
    confirm-tx              Confirms successful transaction
    connect                 Sets node rest API address. Verifies connection on set
    funds                   Prints information about voting funds
    help                    Prints this message or the help of the given subcommand(s)
    logs                    Prints entire fragment logs from the node
    pending-transactions    Prints pending transactions (not confirmed)
    proposals               Prints proposals available to vote on
    refresh                 Pulls wallet data from the catalyst backend
    status                  Prints wallet status (balance/spending counters/tokens)
    statuses                Prints pending or already sent fragments statuses
    vote                    Sends votes to backend
    votes                   Prints history of votes
    wallets                 Allows to manage wallets: add/remove/select operations
```
