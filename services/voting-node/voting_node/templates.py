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

GENESIS_YAML_TEMPLATE: Final = """
# The Blockchain Configuration defines the settings of the blockchain.
blockchain_configuration:

  # The block0-date defines the date the blockchain starts
  # expected value in seconds since UNIX_EPOCH
  #
  # By default the value will be the current date and time. Or you can
  # add a specific time by entering the number of seconds since UNIX
  # Epoch
  block0_date: 1677725933

  # This is the type of discrimination of the blockchain
  # if this blockchain is meant for production then
  # use 'production' instead.
  #
  # otherwise leave as this
  discrimination: test

  # The initial consensus version:
  #
  # * BFT consensus: bft
  # * Genesis Praos consensus: genesis
  block0_consensus: bft

  # Number of slots in each epoch.
  #
  # default value is 720
  slots_per_epoch: 720

  # The slot duration, in seconds, is the time between the creation
  # of 2 blocks
  #
  # default value is 5s
  slot_duration: 5s

  # set the block content max size
  #
  # This is the size, in bytes, of all the contents of the block (excluding the
  # block header).
  #
  # default value is 102400
  block_content_max_size: 102400

  # A list of Ed25519 PublicKey that represents the
  # BFT leaders encoded as bech32. The order in the list matters.
  consensus_leader_ids:
    - ed25519_pk1vvwp2s0n5jl5f4xcjurp2e92sj2awehkrydrlas4vgqr7xzt33jsadha32
    - ed25519_pk1h6k6qcfxc7xe3d9p560kaescj620par4z5ud4qj0rtwgk99pk43qvedxvr

  # Epoch stability depth
  #
  # Optional: default value 102400
  epoch_stability_depth: 102400

  # Genesis praos active slot coefficient
  # Determines minimum stake required to try becoming slot leader, must be in range (0,1]
  #
  # default value: 0.100
  consensus_genesis_praos_active_slot_coeff: 0.100

  # The fee calculations settings
  #
  # total fees: constant + (num_inputs + num_outputs) * coefficient [+ certificate]
  linear_fees:
    # this is the minimum value to pay for every transaction
    constant: 2
    # the additional fee to pay for every inputs and outputs
    coefficient: 1
    # the additional fee to pay if the transaction embeds a certificate
    certificate: 4
    # (optional) fees for different types of certificates, to override the one
    # given in `certificate` just above
    #
    # here: all certificate fees are set to `4` except for pool registration
    # and stake delegation which are respectively `5` and `2`.
    per_certificate_fees:
      # (optional) if not specified, the pool registration certificate fee will be
      # the one set by linear_fees.certificate
      certificate_pool_registration: 5
      # (optional) if not specified, the delegation certificate fee will be
      # the one set by linear_fees.certificate
      certificate_stake_delegation: 2
      # (optional) if not specified, the owner delegation certificate fee will be
      # the one set by linear_fees.certificate. Uncomment to set the owner stake
      # delegation to `1` instead of default `4`:
      # certificate_owner_stake_delegation: 1

  # Proposal expiration in epochs
  #
  # default value: 100
  proposal_expiration: 100

  # The speed to update the KES Key in seconds
  #
  # default value: 12h
  kes_update_speed: 12h

  # Set where to send the fees generated by transactions activity.
  #
  # by default it is send to the "rewards" pot of the epoch which is then
  # distributed to the different stake pools who created blocks that given
  # epoch.
  #
  # It is possible to send all the generated fees to the "treasury".
  #
  # Optional, default is "rewards"
  # fees_go_to: "rewards"

  # initial value the treasury will start with, if not set the treasury
  # starts at 0
  treasury: 1000000000000

  # set the treasury parameters, this is the tax type, just as in stake pool
  # registration certificate parameters.
  #
  # When distributing the rewards, the treasury will be first serve as per
  # the incentive specification document
  #
  # if not set, the treasury will not grow
  treasury_parameters:
    # the fix value the treasury will take from the total reward pot of the epoch
    fixed: 1000
    # the extra percentage the the treasury will take from the reward pot of the epoch
    ratio: "1/10"
    # It is possible to add a max bound to the total value the treasury takes
    # at each reward distribution. For example, one could cap the treasury tax
    # to 10000. Uncomment the following line to apply a max limit:
    # max_limit: 10000

  # Set the total reward supply available for monetary creation
  #
  # if not set there is no monetary creation
  # once emptied, there is no more monetary creation
  total_reward_supply: 100000000000000

  # set the reward supply consumption. These parameters will define how the
  # total_reward_supply is consumed for the stake pool reward
  #
  # There's fundamentally many potential choices for how rewards are contributed back, and here's two potential valid examples:
  #
  # Linear formula: constant - ratio * (#epoch after epoch_start / epoch_rate)
  # Halving formula: constant * ratio ^ (#epoch after epoch_start / epoch_rate)
  #
  reward_parameters:
    halving: # or use "linear" for the linear formula
      # In the linear formula, it represents the starting point of the contribution
      # at #epoch=0, whereas in halving formula is used as starting constant for
      # the calculation.
      constant: 100

      # In the halving formula, an effective value between 0.0 to 1.0 indicates a
      # reducing contribution, whereas above 1.0 it indicate an acceleration of contribution.
      #
      # However in linear formula the meaning is just a scaling factor for the epoch zone
      # (current_epoch - start_epoch / epoch_rate). Further requirement is that this ratio
      # is expressed in fractional form (e.g. 1/2), which allow calculation in integer form.
      ratio: "13/19"

      # indicates when this contribution start. note that if the epoch is not
      # the same or after the epoch_start, the overall contribution is zero.
      epoch_start: 1

      # the rate at which the contribution is tweaked related to epoch.
      epoch_rate: 3

  # set some reward constraints and limits
  #
  # this value is optional, the default is no constraints at all. The settings
  # are commented below:
  #
  #reward_constraints:
  #  # limit the epoch total reward drawing limit to a portion of the total
  #  # active stake of the system.
  #  #
  #  # for example, if set to 10%, the reward drawn will be bounded by the
  #  # 10% of the total active stake.
  #  #
  #  # this value is optional, the default is no reward drawing limit
  #  reward_drawing_limit_max: "10/100"
  #
  #  # settings to incentivize the numbers of stake pool to be registered
  #  # on the blockchain.
  #  #
  #  # These settings does not prevent more stake pool to be added. For example
  #  # if there is already 1000 stake pools, someone can still register a new
  #  # stake pool and affect the rewards of everyone else too.
  #  #
  #  # if the threshold is reached, the pool doesn't really have incentive to
  #  # create more blocks than 1 / set-value-of-pools % of stake.
  #  #
  #  # this value is optional, the default is no pool participation capping
  #  pool_participation_capping:
  #    min: 300
  #    max: 1000

  # list of the committee members, they will be used to guarantee the initial
  # valid operation of the vote as well as privacy.
  committees:
    - "7ef044ba437057d6d944ace679b7f811335639a689064cd969dffc8b55a7cc19"
    - "f5285eeead8b5885a1420800de14b0d1960db1a990a6c2f7b517125bedc000db"

# Initial state of the ledger. Each item is applied in order of this list
initial:
  # Initial deposits present in the blockchain
  - fund:
      # UTxO addresses or account
      - address: ca1s5s0mwkfky9htpam576mc93mee5709khre8dgnqslj6y3p5f77s5g0cflq8
        value: 10000
      - address: ca1s467g96d6kyzy4yqsmchkqmuajcwapj3dd73xsq2sry9dd9a4al7z8xnex8
        value: 10000
  # Initial token distribution
  - token:
      token_id: 00000000000000000000000000000000000000000000000000000000.7e5d6abc
      to:
        - address: ca1s5s0mwkfky9htpam576mc93mee5709khre8dgnqslj6y3p5f77s5g0cflq8
          value: 150
        - address: ca1s467g96d6kyzy4yqsmchkqmuajcwapj3dd73xsq2sry9dd9a4al7z8xnex8
          value: 255
  - token:
      token_id: 00000000000000000000000000000000000000000000000000000000.6c1e8abc
      to:
        - address: ca1s5s0mwkfky9htpam576mc93mee5709khre8dgnqslj6y3p5f77s5g0cflq8
          value: 22
        - address: ca1s467g96d6kyzy4yqsmchkqmuajcwapj3dd73xsq2sry9dd9a4al7z8xnex8
          value: 66

  # Initial certificates
  #- cert: ..

  # Initial deposits present in the blockchain
  #- legacy_fund:
  #    # Legacy Cardano address
  #    - address: 48mDfYyQn21iyEPzCfkATEHTwZBcZJqXhRJezmswfvc6Ne89u1axXsiazmgd7SwT8VbafbVnCvyXhBSMhSkPiCezMkqHC4dmxRahRC86SknFu6JF6hwSg8
  #      value: 123

"""
