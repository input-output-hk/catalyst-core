# Configuration files go there


from typing import Final


class JormConfig(object):
    """Holds parameters used to configure and start jormungandr."""

    def __init__(
        self,
        jormungandr_path: str,
        jcli_path: str,
        storage: str,
        rest_port: int,
        jrpc_port: int,
        p2p_port: int,
    ):
        self.jormungandr_path = jormungandr_path
        self.jcli_path = jcli_path
        self.storage = storage
        self.rest_port = rest_port
        self.jrpc_port = jrpc_port
        self.p2p_port = p2p_port


NODE_CONFIG_TEMPLATE: Final = """
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
