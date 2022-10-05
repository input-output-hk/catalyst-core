## Configuration

This section describe configuration file which can be passed as argument for registration verify service:

- `port`: port on which registration-verify-service will be exposed,
- `jcli`: path to jcli executable,
- `snapshot-token`: token required by [snapshot-service](../snapshot-service/introduction.md),
- `snapshot-address`: address of [snapshot-service](../snapshot-service/introduction.md),
- `client-token`: access token for client endpoints (verifying voting power etc.),
- `admin-token`: access token for admin endpoints (updating snapshot etc.),,
- `network`: network type. Possible values: 
   - `mainnet`
   - `{ "testnet": 1097911063 }`,
- `initial-snapshot-job-id`: initial job id from snapshot service that will be used when starting service
Example:

```yaml
    "port": 8080,
    "jcli": "jcli",
	"snapshot-token": "3568b599a65557b2a2e",
	"snapshot-address": "https://snapshot.address:8080",
	"client-token": "5e19639accf2d76bae",
	"admin-token": "605a7c515ec781fd39",
	"network": "mainnet",
	"initial-snapshot-job-id": "3b49a0ae-5536-454b-8f47-780d9e7da6a0"
```