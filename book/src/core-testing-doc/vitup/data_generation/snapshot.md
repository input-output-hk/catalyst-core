## Configuration

This section describe configuration section which can be passed as argument when starting vitup or
 send to already running environments in order to restart them through rest api.

### Example

```json
{
    "parameters": {
        "tag": "latest"
    },
    "content": [
        {
            "rep_name": "alice",
            "ada": 1000
        },
        {
            "rep_name": "clarice",
            "ada": 1000
        },
        {
            "name": "bob",
            "registration": {
                "target": [
                    ["alice",1]
                ],
                "slotno": 0
            },            
            "ada": 1000
        },
         {
            "name": "david",
            "registration": {
                "target": [
                    ["clarice",1]
                ],
                "slotno": 0
            },            
            "ada": 1000
        }
    ]

```

Below more detailed explanation for each section element

### parameters

Snapshot parameters used when importing it to servicing station or mock.

- `tag` - snapshot tag which will be used when importing snapshot
- `min_stake_threshold` - Minimum lovelace which is required to participate in voting
- `voting_power_cap` - Maximum percentage of voting power before capping
- `direct_voters_group` - Name of direct registration holders
- `representatives_group` - Name of delegated registrations holders (representatives)

### content

Main content of snapshot

#### actor

For user convenience we allow untagged definition of actor. Actor can be representative or direct voter with some data.
Depending on fields role is dynamically defined and user can focus only on scenario description

##### pre-generated representative

This variant will create new unique wallet with given ada amount

- `rep_name` - alias
- `ada` - voting power amount

##### external representative

Representative with just and voting key. Can be used for already existing wallet

- `rep_name` - alias
- `voting_key` - voting key in hex

##### external delegator

Delegator with just an address. Can be used for already existing wallet in the network

- `name` - alias
- `address` - address in hex

##### pre-generated delegator

Delegator with just an address. Can be used for already existing wallet in the network.
Generated delegator will set up new mainnet wallet

`name` - alias

`registration`: registration definition which can be used to describe to which representative delegator delegates his voting power.
                Field need to define slot at which delegation occurs and distribution. Example:

```yaml
...
  "registration": {
   "target": [ [ "clarice",1 ] ,[ "alice",2 ] ],
   "slotno": 0
  }
...
```

Above example divides voting power into 3 parts and assign 1/3 to clarice and 2/3 to alice

`ada` - ada amount
