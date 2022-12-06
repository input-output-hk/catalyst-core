## Configuration

This section describe configuration file which can be passed as argument when starting snapshot-wormhole:

### snapshot service

This section describe snapshot trigger service connection:

- `address`: snapshot trigger REST api address,
- `token`: optional access token,

### servicing station service

This section describe servicing station service connection:

- `address`: servicing station service REST api address,,

### parameters

This section defines snapshot import parameters when applying snapshot to vit servicing station

- `min_stake_threshold`: minimum stake needed to participate in voting. Expressed in ada,
- `voting_power_cap`: maximum voting power before capping in order to satisfy fairness in voting. Expressed as a fraction number,
- `direct_voters_group`: group name for direct voters (determines part of REST path when accessing particular group with GET request),
- `representatives_group`: group name for representatives (determines part of REST path when accessing particular group with GET request)

Example:

```yaml
{
    "snapshot_service": {
        "address": "http://127.0.0.1:9090",
        "token": "RBj0weJerr87A"
    },
    "servicing_station": {
        "address": "http://127.0.0.1:8080"
     },
    "parameters": {
        "min_stake_threshold": 500,
        "voting_power_cap": {
		    "Rational": ["Plus",[1,2]]
        },
        "direct_voters_group": "direct",
        "representatives_group": "rep"
    }
}
"
```