# Registration service

Registration service is REST service purely for test purposes which is capable to interact with [voter registration cli](https://github.com/input-output-hk/voting-tools/tree/master/registration), [cardano cli](https://github.com/input-output-hk/cardano-node/tree/master/cardano-cli) and [vit-kedqr](https://github.com/input-output-hk/vit-kedqr).

## build

In order to build registration-verify-service in main project folder run:

```
cd registration-verify-service
cargo build
cargo install --path . --force
```

## quick start

The simplest configuration is available by using command:

`registration-service --config config.yaml`

See [config](./configuration.md) for more details.

## clients

### cli

Registration CLI is cli utility tool which help to interact with registration verify service without manually constructing requests

See [cli](./cli.md) for more details.

### api

Example:

```
    use registration_verify_service::client::rest::RegistrationVerifyRestClient;

    ...
    
    let registration_verify_client =
        RegistrationVerifyRestClient::new_with_token(registration_token, registration_address);

     let mut form = Form::new()
            .text("pin", "1234")
            .text("funds","500")
            .text("threshold", "500")
            .file("qr", PathBuf::new("my_q.png")?;

    let registration_job_id = registration_verify_client.job_new(form).unwrap();

    let wait = WaitBuilder::new().tries(10).sleep_between_tries(10).build();
    println!("waiting for registration job");
    let registration_jobs_status = registration_client
        .wait_for_job_finish(registration_job_id.clone(), wait)
        .unwrap();
    println!("{:?}", registration_jobs_status);
```
