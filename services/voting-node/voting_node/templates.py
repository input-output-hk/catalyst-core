from typing import Final

"""Default template for node_config.yaml."""
NODE_CONFIG_LEADER0_TEMPLATE: Final = """
---
storage: #"./node_storage"
rest:
  listen: #"127.0.0.1:10003"
jrpc:
  listen: #"127.0.0.1:10005"
p2p:
  bootstrap:
    trusted_peers: []
    node_key_file: #"./node_storage/node_topology_key"
  connection:
    public_address: #/ip4/127.0.0.1/tcp/10004
    allow_private_addresses: true
    whitelist: ~
  policy:
    quarantine_duration: 1s
  layers:
    topics_of_interest:
      messages: high
      blocks: high
log:
  format: json
  level: DEBUG
  output: stdout
mempool:
  pool_max_entries: 10000
  log_max_entries: 100000
  persistent_log: ~
bootstrap_from_trusted_peers: false
skip_bootstrap: true
"""

NODE_CONFIG_LEADER_TEMPLATE: Final = """
---
storage: # "./catalyst/quick/leader2/storage"
rest:
  listen: # "127.0.0.1:10000"
jrpc:
  listen: # "127.0.0.1:10002"
p2p:
  bootstrap:
    trusted_peers:
      - address: # /ip4/127.0.0.1/tcp/10004
      - address: # /ip4/127.0.0.1/tcp/10001
    node_key_file: # "./catalyst/quick/leader2/node_topology_key"
  connection:
    public_address: # /ip4/127.0.0.1/tcp/10001
    allow_private_addresses: true
    whitelist: ~
  policy:
    quarantine_duration: 1s
  layers:
    topics_of_interest:
      messages: high
      blocks: high
log:
  format: json
  level: DEBUG
  output: stdout
mempool:
  pool_max_entries: 10000
  log_max_entries: 100000
  persistent_log: ~
bootstrap_from_trusted_peers: true
skip_bootstrap: false
"""

NODE_CONFIG_FOLLOWER_TEMPLATE: Final = """
---
storage: # "./catalyst/quick/follower/storage"
rest:
  listen: # "127.0.0.1:10006"
jrpc:
  listen: # "127.0.0.1:10010"
p2p:
  bootstrap:
    trusted_peers:
      - address: # /ip4/127.0.0.1/tcp/10001
      - address: # /ip4/127.0.0.1/tcp/10004
      - address: # /ip4/127.0.0.1/tcp/10012
    node_key_file: # "./catalyst/quick/follower/node_topology_key"
  connection:
    public_address: # /ip4/127.0.0.1/tcp/10009
    allow_private_addresses: true
    whitelist: ~
  policy:
    quarantine_duration: 1s
  layers:
    topics_of_interest:
      messages: high
      blocks: high
log:
  format: json
  level: DEBUG
  output: stdout
mempool:
  pool_max_entries: 1000000
  log_max_entries: 100000
  persistent_log:
    dir: # "./catalyst/persistent_log"
bootstrap_from_trusted_peers: true
skip_bootstrap: false
"""
