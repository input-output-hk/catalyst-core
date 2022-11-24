## Configuration

This section describe configuration file which can be passed as argument when starting valgrind:

- `address`: address on which valgrind will be exposed. By default: `127.0.0.1:8000`,
- `vit-address`: vit servicing station address. By default: `127.0.0.1:3030`,
- `node-address`: node address.  By default: `127.0.0.1:8080`,
- `block0-path`: path to block0 executable,
- `cert`: path to certificate (for enabling https). Optional,
- `key`: path certificate key (for enabling https). Optional,


Example:

```yaml
   "address": "127.0.0.1:8000",
	"vit-address": "127.0.0.1:3030",
	"node-address": "127.0.0.1:8080",
	"block0-path": "./block0.bin",
	"cert": "certificate.cert",
	"key": "certificate.key",
"
```

