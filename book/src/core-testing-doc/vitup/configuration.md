
## Configuration

Configuration file example is available under `src/vit-testing/vitup/example/mock/config.yaml`
This section describe configuration file which can be passed as argument for `vitup start mock` command:

pub struct Configuration {

pub ideascale: bool,

pub protocol: valgrind::Protocol,

# [serde(default)]

pub local: bool,
}

- `port`: port on which registration-service will be exposed,
- `token`: token limiting access to environment. Must be provided in header `API-Token` for each request
- `working-dir`: path to folder which artifacts will be dumped (qr-code etc.),
- `protocol`: optional parameter if service shoudl be exposed through https. Then two sub-parameters need to be defined `
              key_path` and `cert_path` like in an example below:

    ```
      "protocol": {
        "key_path": "./resources/tls/server.key",
        "cert_path": "./resources/tls/server.crt"
      }
    ```

    NOTE: certificates in resources folder are self-signed

Example:

```yaml
{
  "port": 8080,
  "working-dir": "./mock",
  "protocol": {
    "key_path": "./resources/tls/server.key",
    "cert_path": "./resources/tls/server.crt"
  }
}
```
