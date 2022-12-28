# Automatic deployment of the voting blockchain

Originally the voting blockchain was designed to be manually started and
required a full block 0 and a configuration file to be created and distributed
to nodes before it could commence.

This made automated deployment difficult and introduces necessary manual steps
into the process of running the voting system.

To resolve this,  the voting system is modified to allow the blockchain and
parts of the configuration to be automatically created based solely on the
parameters of the next election.

## Overview

There are two sources of data required to start the blockchain.  Block 0 and
the config YAML file. To ease deployment,  Block 0 will be created dynamically
based on data held in our distributed object storage (which is currently a
Postgresql Database.)  As are certain parameters currently required for the
configuration file.

The blockchain would still need to retain the current method for starting, in
addition to the new "auto" mode.

In essence, automatic configuration entails:

1. Minimizing manual config items to only those that unavoidably need to be
   defined.
2. Generating configuration for other items where possible from known local
   state, and only having configuration items for these to override the defaults.
3. Sharing data in a central repository of local configuration items that other
   nodes would require.
4. Reading other data directly from their source of truth (Such as the schedule
   of the election, voting power snapshot data and proposals/vote plan
   information.)

## Configuration

The node is configured by a YAML file which contains the following data. In the
code, every config parameter should be accompanied by detailed a detailed
documentation comment.

* `secret_file:` - Optional Path (to what, used for what?)
* `storage:` - Optional Path (to what, used for what?)
* `log:` - Optional Logger settings.
  * `level:` - Optional Logger level, can be `"Trace"`, `"Debug"`, `"Info"`.
    `"Warn"` and `"Error"`.  Should default to `"Info"` if not set.
  * `format:` - Format of the logs, can be `"plain"` and `"json"`.  Should
    default to `"json"` if not set.
  * `output:` - Optional destination of the log output. *Options need to be
    fully documented*.  Should default to `stdout` if not defined.
  * `trace_collector_endpoint:` - Optional *Options need to be fully
    documented*.  Should default to None (ie, no external logging) if not
    defined.
* `mempool:` Optional configuration of the mempool.  Should default as specified
  here.
  * `pool_max_entries:` - Optional - maximum number of entries in the mempool.
    Should default to 1,000,000 if not set.
  * `log_max_entries:` - Optional - maximum number of entries in the fragment
    logs.  Should default to ???? if not set.
  * `persistent_log:` - Optional - path to the persistent log of all incoming
    fragments.  A decision needs to be made if persistent logging is normally
    desired.  If it is, it should default to a location in `/var`.  If not, it
    should default to None and be disabled.
* `leadership:` - Optional - the number of entries allowed in the leadership
  logs.
  * `logs_capacity:` - Optional - Should default to ???? if not set.
* `rest:` - Optional - Enables REST API.
  * `listen:` - Optional - Address to listen to rest api requests on.  Should
    default to  "0.0.0.0:12080".  *This default is open to suggestions*
  * `tls:`  - Optional - Define inbuilt tls support for the listening socket.
    If not specified, TLS is disabled.  The default is TLS Disabled.
    * `cert_file:` - Path to server X.509 certificate chain file, must be
      PEM-encoded and contain at least 1 item
    * `priv_key_file:` - Path to server private key file, must be PKCS8 with
      single PEM-encoded, unencrypted key
  * `cors:` - Optional - Defines CORS settings.  Default should be as shown in
    the individual entries.
    * `allowed_origins` - Origin domains we accept connections from. Defaults to
      "*".
    * `max_ages_secs` - How long in seconds to cache CORS responses.  Defaults
      to 60.
    * `allowed_headers` - A list of allowed headers in the preflight check.  If
      the provided list is empty, all preflight requests with a request header
      will be rejected. Default should be a value which allows cors to work
      without requiring extra config under normal circumstances.
    * `allowed_methods` - A list of allowed methods in the preflight check.  If
      the provided list is empty, all preflight requests will be rejected.
      Default should be a value which allows cors to work without requiring
      extra config under normal circumstances.
* `jrpc:` - Optional.

    #[serde(default)]
    pub leadership: Leadership,

    pub rest: Option<Rest>,

    pub jrpc: Option<JRpc>,

    #[serde(default)]
    pub p2p: P2pConfig,

    #[serde(default)]
    pub http_fetch_block0_service: Vec<String>,

    #[cfg(feature = "prometheus-metrics")]
    pub prometheus: Option<Prometheus>,

    /// the time interval with no blockchain updates after which alerts are thrown
    #[serde(default)]
    pub no_blockchain_updates_warning_interval: Option<Duration>,

    #[serde(default)]
    pub bootstrap_from_trusted_peers: bool,

    #[serde(default)]
    pub skip_bootstrap: bool,

    pub block_hard_deadline: Option<u32>,
