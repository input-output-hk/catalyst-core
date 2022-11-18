## Configuration

This section describe configuration file which can be passed as argument for registration service:

- `port`: port on which registration-service will be exposed,
- `jcli`: path to jcli executable,
- `result-dir`: path to folder which artifacts will be dumped (qr-code etc.),
- `cardano-cli`: path to jcli executable,
- `voter-registration`: path to jcli executable,
- `vit-kedqr`: path to jcli executable,
- `network`: network type. Possible values: 
   - `mainnet`
   - `{ "testnet": 1097911063 }`,
- `token`: token limiting access to environment. Must be provided in header `API-Token` for each request

Example:

```yaml
  "port": 8080,
	"jcli": "jcli",
	"result-dir": "/persist",
	"cardano-cli": "./cardano-cli",
	"voter-registration": "./voter-registration",
	"vit-kedqr": "./vit-kedqr",
	"network": "mainnet",
	"token": "..."
```
