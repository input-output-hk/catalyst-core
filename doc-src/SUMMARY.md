# Summary

[Introduction](./intro.md)

- [Vote Storage Ledger](./core-ledger-doc/introduction.md)

  - [General Concepts](./core-ledger-doc/concepts/introduction.md)
    - [Blockchain concepts](./core-ledger-doc/concepts/blockchain.md)
    - [Stake](./core-ledger-doc/concepts/stake.md)
    - [Node organisation](./core-ledger-doc/concepts/node.md)
    - [Network overview](./core-ledger-doc/concepts/network.md)

  - [Quick start](./core-ledger-doc/quickstart/introduction.md)
    - [Command lines tools](./core-ledger-doc/quickstart/01_command_line.md)
    - [Starting a passive node](./core-ledger-doc/quickstart/02_passive_node.md)
    - [REST API](./core-ledger-doc/quickstart/03_rest_api.md)
    - [Explorer API](./core-ledger-doc/quickstart/04_explorer.md)
    - [Starting as a leader candidate](./core-ledger-doc/quickstart/05_leader_candidate.md)

  - [Configuration](./core-ledger-doc/configuration/introduction.md)
    - [Logging](./core-ledger-doc/configuration/logging.md)
    - [Node network](./core-ledger-doc/configuration/network.md)
    - [Fragment Pool](./core-ledger-doc/configuration/mempool.md)
    - [Leader Events](./core-ledger-doc/configuration/leadership.md)

  - [jcli](./core-ledger-doc/jcli/introduction.md)
    - [Cryptographic keys](./core-ledger-doc/jcli/key.md)
    - [Address](./core-ledger-doc/jcli/address.md)
    - [Transaction](./core-ledger-doc/jcli/transaction.md)
    - [Certificate](./core-ledger-doc/jcli/certificate.md)
    - [Genesis](./core-ledger-doc/jcli/genesis.md)
    - [Voting](./core-ledger-doc/jcli/vote.md)
    - [REST](./core-ledger-doc/jcli/rest.md)

  - [Staking and stake pool](./core-ledger-doc/stake_pool/introduction.md)
    - [Delegating your stake](./core-ledger-doc/stake_pool/delegating_stake.md)
    - [Registering stake pool](./core-ledger-doc/stake_pool/registering_stake_pool.md)
    - [Retiring stake pool](./core-ledger-doc/stake_pool/retiring_stake_pool.md)

  - [Advanced](./core-ledger-doc/advanced/introduction.md)
    - [Genesis block](./core-ledger-doc/advanced/01_the_genesis_block.md)
    - [Starting a bft blockchain](./core-ledger-doc/advanced/02_starting_bft_blockchain.md)
    - [Starting a genesis blockchain](./core-ledger-doc/advanced/03_starting_genesis_praos_blockchain.md)

  - [Specs](./core-ledger-doc/specs/introduction.md)
    - [Network](./core-ledger-doc/specs/network.md)

  - [Testing](./core-ledger-doc/testing/introduction.md)
    - [Jormungandr Automation](./core-ledger-doc/testing/automation.md)
    - [Hersir](./core-ledger-doc/testing/hersir.md)
    - [Thor](./core-ledger-doc/testing/thor.md)
    - [Mjolnir](./core-ledger-doc/testing/mjolnir.md)
    - [Loki](./core-ledger-doc/testing/loki.md)
    - [Integration tests](./core-ledger-doc/testing/integration_tests.md)

- [Rust Documentation](./rust/docs.md)

- [API Documentation](./api/api.md)
  - [Jormungandr V0 Rest API](./api/JormungandrV0.md)
  - [Jormungandr V1 Rest API](./api/JormungandrV1.md)
  - [VIT Servicing Station Rest API](./api/vit-servicing-station-v0.md)
  - [VIT Testing Rest API](./api/vit-testing-v0.md)
