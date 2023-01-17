# Snapshot trigger service

Service which operates on top of [voting tools](https://github.com/input-output-hk/voting-tools) and is a interface improvement which expose voting tools as a REST service.

## build

In order to build snapshot-trigger-service in main project folder run:

```
cd snapshot-trigger-service
cargo build
cargo install --path . --force
```

## quick start

The simplest configuration is available by using command:

`snapshot-trigger-service --config config.yaml`

See [config](./configuration.md) for more details.

### Usage

In order to start new job one need to send POST request like below:

```
curl --location --request POST 'https://snapshot.io/api/job/new' \
--header 'API-Token: ...' \
--header 'Content-Type: application/json' \
--data-raw '{
    "threshold": 2000000, // IN Lovelace
    "slot-no": 31842935
}'
```

Response will contains job status:
`b0b7b774-7263-4dce-a97d-c167169c8f27`

Then query for job status:

```
curl --location --request GET 'https://snapshot.io/api/job/status/b0b7b774-7263-4dce-a97d-c167169c8f27' \
--header 'API-Token: ...'
```

and finally fetch snapshot:

```
curl --location --request GET 'https://snapshot.io/api/job/files/get/b0b7b774-7263-4dce-a97d-c167169c8f27/snapshot.json' \
--header 'API-Token: ...'
```

which has form:

```
{
    "initial": [
        {
            "fund": [
                {
                    "address": "ca1q5yr504t56ruuwrp5zxpu469t9slk0uhkefc7admk7wqrs24q6nxwyhwjcf",
                    "value": 14463
                },
                {
                    "address": "ca1q5ynl2yqez8lmuaf3snvgcw885c9hxxq6uexeevd4al8pct7vx69sljvzxe",
                    "value": 9991
                },
....
```

## clients

### cli

Snapshot CLI is cli utility tool which help to interact with snapshot trigger service without manually constructing requests

See [cli](./cli.md) for more details.

### api

Example:

```
    use snapshot_trigger_service::{
        client::rest::SnapshotRestClient,
        config::JobParameters,
        State,
    };

    let job_param = JobParameters {
        slot_no: Some(1234567),
        tag: Some("fund1".to_string()),
    };

    let snapshot_token=  "...";
    let snapshot_address =  "...";

    let snapshot_client =
        SnapshotRestClient::new_with_token(snapshot_token.into(), snapshot_address.into());

    let snapshot_job_id = snapshot_client.job_new(job_params).unwrap();
    let wait = WaitBuilder::new().tries(10).sleep_between_tries(10).build();

    let snapshot_jobs_status =
        snapshot_client.wait_for_job_finish(snapshot_job_id.clone(), wait)?;

    let snapshot = snapshot_client.get_snapshot(snapshot_job_id)?;
```
