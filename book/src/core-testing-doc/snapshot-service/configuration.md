## Configuration

This section describe configuration file which can be passed as argument for snapshot service:

- `port`: port on which registration-service will be exposed,
- `result-dir`: path to folder which artifacts will be dumped (qr-code etc.),
- `voting-tools`: voting tools internal parameters section,
  - `bin`: "voting-tools",
  - `network`: network type. Possible values: 
      - `mainnet`
      - `{ "testnet": 1097911063 }`,
	  -	`db`: dbsync name,
	  -	`db-user`: dbsync user,
	  -	`db-host`: dbsync host,
	  -	`scale`: voting power multiplier. If 1 then Lovelace is used
- `voter-registration`: path to jcli executable,
- `vit-kedqr`: path to jcli executable,

- `token`: token limiting access to environment. Must be provided in header `API-Token` for each request

Example:

```yaml
 "port": 8080,
	"result-dir": "/persist/snapshot",
	"voting-tools": {
		    "bin": "voting-tools",
		    "network": "mainnet",
		    "db": "dbsync",
		    "db-user": "dbsync-admin",
		    "db-host": "/alloc",
		    "scale": 1000000
	},
	"token": "3568b599a65557b2a2e"
```

