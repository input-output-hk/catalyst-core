from typing import Final

"""Default template for node_config.yaml."""
NODE_CONFIG_LEADER0: Final = """
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

NODE_CONFIG_LEADER: Final = """
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

NODE_CONFIG_FOLLOWER: Final = """
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

GENESIS_YAML: Final = """
---
blockchain_configuration:
  block0_date: 1675728877
  discrimination: production
  block0_consensus: bft
  consensus_leader_ids:
    - ed25519_pk1m7v0ndkmhw43xu22rg9yc4em3kcpc5zjmzt4mppxwwszdppk572smfw60m
    - ed25519_pk14gvq29tpgd8rv6fhal63uqhsne5l94c2gjd6k3624u6qpvt06wks35w7f3
    - ed25519_pk1yjs2kx293c9xz5evs0g66r0eaffztgufrawpnt9qylysykzn9veswte8g6
  linear_fees:
    constant: 0
    coefficient: 0
    certificate: 0
  proposal_expiration: 100
  slots_per_epoch: 60
  slot_duration: 20
  kes_update_speed: 46800
  consensus_genesis_praos_active_slot_coeff: "0.500"
  block_content_max_size: 102400
  epoch_stability_depth: 102400
  tx_max_expiry_epochs: 2
  treasury: 1000000
  committees:
    - a905194f3dfb41d03b81671b4aeae0fa74924f6088e4b2129a7aeb6acd5df6c2
initial:
  - fund:
      - address: ca1q4ccde0ax2yv7yx7t7uy6v94yd95pc2u5nrs6wkcn660xp2pj7w2g29q23a
        value: 9508
  - token:
      token_id: 00000000000000000000000000000000000000000000000000000000.80e38c705071df24015513f5c19d7dc1077cf9943440a8120c5011c64c888cda
      to:
        - address: ca1q4ccde0ax2yv7yx7t7uy6v94yd95pc2u5nrs6wkcn660xp2pj7w2g29q23a
          value: 9508
  - fund:
      - address: ca1qh5m6qu6nq7ldg8gxj6nqnq4432y9pr5h59dkf0zntm97m72rvxl6ywaq5z
        value: 12644
  - token:
      token_id: 00000000000000000000000000000000000000000000000000000000.80e38c705071df24015513f5c19d7dc1077cf9943440a8120c5011c64c888cda
      to:
        - address: ca1qh5m6qu6nq7ldg8gxj6nqnq4432y9pr5h59dkf0zntm97m72rvxl6ywaq5z
          value: 12644
  - fund:
      - address: ca1q4w5r3mj6mtuqngk2hrr9eeahc23xu5r9xlfak6dxjke4jwxrah75uuv0zd
        value: 9309
  - token:
      token_id: 00000000000000000000000000000000000000000000000000000000.80e38c705071df24015513f5c19d7dc1077cf9943440a8120c5011c64c888cda
      to:
        - address: ca1q4w5r3mj6mtuqngk2hrr9eeahc23xu5r9xlfak6dxjke4jwxrah75uuv0zd
          value: 9309
  - fund:
      - address: ca1qhmhn0gdfjp4y32cz0tng70r3gdtqgckqyvruyqpccsrvvyvlmsmyeuv374
        value: 11350
  - token:
      token_id: 00000000000000000000000000000000000000000000000000000000.80e38c705071df24015513f5c19d7dc1077cf9943440a8120c5011c64c888cda
      to:
        - address: ca1qhmhn0gdfjp4y32cz0tng70r3gdtqgckqyvruyqpccsrvvyvlmsmyeuv374
          value: 11350
  - fund:
      - address: ca1q5fdk35kklrleuj3xl2s0kr743s8yy9vtgyqefmekhnv0yv94g9hvm0kjew
        value: 11011
  - token:
      token_id: 00000000000000000000000000000000000000000000000000000000.80e38c705071df24015513f5c19d7dc1077cf9943440a8120c5011c64c888cda
      to:
        - address: ca1q5fdk35kklrleuj3xl2s0kr743s8yy9vtgyqefmekhnv0yv94g9hvm0kjew
          value: 11011
  - fund:
      - address: ca1q5yddz8w7gv9nm7p4g7dw0v63lpvhuyd9svr5ugakpzg0d6qdr9dvmxm4r7
        value: 12455
  - token:
      token_id: 00000000000000000000000000000000000000000000000000000000.80e38c705071df24015513f5c19d7dc1077cf9943440a8120c5011c64c888cda
      to:
        - address: ca1q5yddz8w7gv9nm7p4g7dw0v63lpvhuyd9svr5ugakpzg0d6qdr9dvmxm4r7
          value: 12455
  - fund:
      - address: ca1q4a5sw6v66yjcayyc05atthej3wxusptjr24jh0zh93zfxj6u6nfuycxq2f
        value: 9984
  - token:
      token_id: 00000000000000000000000000000000000000000000000000000000.80e38c705071df24015513f5c19d7dc1077cf9943440a8120c5011c64c888cda
      to:
        - address: ca1q4a5sw6v66yjcayyc05atthej3wxusptjr24jh0zh93zfxj6u6nfuycxq2f
          value: 9984
  - fund:
      - address: ca1q4u3vjq59s227hedxp6gf6jtmufgs8n0qv8ch88ps094h5rc3593sqzw6f8
        value: 10615
  - token:
      token_id: 00000000000000000000000000000000000000000000000000000000.80e38c705071df24015513f5c19d7dc1077cf9943440a8120c5011c64c888cda
      to:
        - address: ca1q4u3vjq59s227hedxp6gf6jtmufgs8n0qv8ch88ps094h5rc3593sqzw6f8
          value: 10615
  - fund:
      - address: ca1q5cs2nrkel0vl3stusxpuee3qraedk4084f6rcng7pg6txfl0wqj5qvgjvs
        value: 10030
  - token:
      token_id: 00000000000000000000000000000000000000000000000000000000.80e38c705071df24015513f5c19d7dc1077cf9943440a8120c5011c64c888cda
      to:
        - address: ca1q5cs2nrkel0vl3stusxpuee3qraedk4084f6rcng7pg6txfl0wqj5qvgjvs
          value: 10030
  - fund:
      - address: ca1qk5s2x208ha5r5pms9n3kjh2ura8fyj0vzywfvsjnfawk6kdthmvyqam88x
        value: 1000000
  - token:
      token_id: 00000000000000000000000000000000000000000000000000000000.80e38c705071df24015513f5c19d7dc1077cf9943440a8120c5011c64c888cda
      to:
        - address: ca1qk5s2x208ha5r5pms9n3kjh2ura8fyj0vzywfvsjnfawk6kdthmvyqam88x
          value: 1000000
  - fund:
      - address: ca1q4ccwlmef42eaq80hquwuamx40hlggp4nqdrdc8xudkvyf062pgu509k6md
        value: 9609
  - token:
      token_id: 00000000000000000000000000000000000000000000000000000000.80e38c705071df24015513f5c19d7dc1077cf9943440a8120c5011c64c888cda
      to:
        - address: ca1q4ccwlmef42eaq80hquwuamx40hlggp4nqdrdc8xudkvyf062pgu509k6md
          value: 9609
  - cert: signedcert1qcqqqqqpqqqqqqqqqqqqyqqqqqqqqqqqqvqqqqqqqy9xj790kapf32xamdrf76zygqq3nccdzajv0ysl5cn3pa4gzwpsz8czqrm46aa5g5hkkeggtdwwyxrqx8uxhdnaktc8dsc3r2rfu99qejtp7qsq0ksg7dyswf62vxx5rxady5m4heh924hxydtcu6s776hck0xfh0uqyqpku7mp9mxrktmd4xcf854ap6ped9rg84xg63p4xzzpql3fhevqdcpqqwhh05mfw8gft3jdgqteujkxvphsn5jl3lxxlag47lvmh4ulygk9qgqxgdp0r9kldxj99vvghvv5a98ufgq06sfgkknp69ljkrrpncfu6cqzqpwau6sm8vztae59hh97wlqpjga20mj4fshr5wux0tr2vaw8arf5qqsqu6ef8d746ght7k74ptn5c6nnq9e8vfsj4p8w782dpej9hhm3xa4qyq8vncsh9qvczel6eph70vm6863t57rlpfqqfq4xk7uq5slmwvyg8vpqqgss04zw6hgsjrq33y67sx0sq2t22s9q49vz8ak2l0qqe0zj0m04qgqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqsgpcuvwpg8rheyq9238awpn47uzpmulx2rgs9gzgx9qywxfjygek4fq5v5700mg8grhqt8rd9w4c86wjfy7cygujep9xn6ad4v6h0kc2442y33r9xplvk22dp80p9s5e7uu57jcq2vq6jewnpa6ze43knl7hh0rt4v7dg9v4aepr79s8scwsfyrz6uu29lc6d47s8xmt4dtrqpd4r5tv
  - cert: signedcert1qcqqqqqpqqqqqqqqqqqqyqqqqqqqqqqqqvqqqqqqqy9xj790kapf32xamdrf76zygqq3nccdzajv0ysl5cn3pa4gzwpsz8czqrm46aa5g5hkkeggtdwwyxrqx8uxhdnaktc8dsc3r2rfu99qejtp7qsq0ksg7dyswf62vxx5rxady5m4heh924hxydtcu6s776hck0xfh0uqyqpku7mp9mxrktmd4xcf854ap6ped9rg84xg63p4xzzpql3fhevqdcpqqwhh05mfw8gft3jdgqteujkxvphsn5jl3lxxlag47lvmh4ulygk9qgqxgdp0r9kldxj99vvghvv5a98ufgq06sfgkknp69ljkrrpncfu6cqzqpwau6sm8vztae59hh97wlqpjga20mj4fshr5wux0tr2vaw8arf5qqsqu6ef8d746ght7k74ptn5c6nnq9e8vfsj4p8w782dpej9hhm3xa4qyq8vncsh9qvczel6eph70vm6863t57rlpfqqfq4xk7uq5slmwvyg8vpqqgss04zw6hgsjrq33y67sx0sq2t22s9q49vz8ak2l0qqe0zj0m04qgqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqs96v3ydqnq06uedn2qe5suyg7wa8tyg23ktu2ndrzjwhpuvafarvafq5v5700mg8grhqt8rd9w4c86wjfy7cygujep9xn6ad4v6h0kcfnywpsmrf4pt7tfgv7cu54xlwxx8jsz77u64fk5942faew6ycmqa7e2c96rxrzv875lqe8kqf7dfrqfwmjvqmuyau8trvz9e3vp2csfs8x9fp
"""
