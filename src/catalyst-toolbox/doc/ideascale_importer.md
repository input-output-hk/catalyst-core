# Ideascale importer

[Ideascale importer](https://github.com/input-output-hk/catalyst-toolbox#ideascale-import) is the tool we use for dumping fund related data. This data is latter massaged and transformed
into proper formats that is later fed into the vit-servicing-station.


## Knows and how's

### Ideascale API endpoints

There are 4 main endpoints that are pinged for getting the data we need:

1. `campaigns/groups`
2. `stages`
3. `campaigns/{challenge_id}/ideas/0/100000`
4. `funnels`

Please refer to the [ideascale api specification](https://a.ideascale.com/api-docs/index.html) to get specific information
for each endpoint.


The query logic work as follows:

1. Queries the challenges (`funnels`).
2. Queries the funds data (`campaigns/groups`).
3. Filter challenges based on fund id (excluding process improvements one {7666})
4. Queries proposals for each challenge (`campaigns/{challenge_id}/ideas/0/100000`)
5. Queries stages (`stages`)

With these ideascale data (challenges, fund, proposals, and stages) we can build the intermediary data.
It is a process of picking, mixing and filtering on that we query before. Actual building code can be
found [here](https://github.com/input-output-hk/catalyst-toolbox/blob/main/src/ideascale/mod.rs) in the `build_*`
functions.

### Custom fields

Proposals in ideascale contain some fields that may depend on fund configuration. So we could be as flexible as possible 
those were abstracted into a [custom matching struct](https://github.com/input-output-hk/catalyst-toolbox/blob/main/src/ideascale/models/custom_fields.rs).
It has a default implementation which was stable till fund7. But it would be desirable to maintain this as a configuration file
that could be saved as part of the auditable fund data.

### Intermediary data 

It is difficult to handle data as coming from ideascale. Usually, will be nested in ways we will not want it and named it
in ways it is not useful. In order to fit the data in the formats we want as output we need to transform it. In this case
managing this transformation solely with serde would be quite convoluted. So, to fix this we have some 
`de` (deserialize structures) that we use to load, parse and transform data incoming from ideascale
and `se` (serialize structures) that are the final form of the output of this tool.