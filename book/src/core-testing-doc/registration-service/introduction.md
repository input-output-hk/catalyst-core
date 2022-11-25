# Registration service 

Registration service is REST service purely for test purposes which is capable to interact with [voter registration cli](https://github.com/input-output-hk/voting-tools/tree/master/registration), [cardano cli](https://github.com/input-output-hk/cardano-node/tree/master/cardano-cli) and [vit-kedqr](https://github.com/input-output-hk/vit-kedqr).

## build

In order to build iapyx in main project folder run:
```
cd registration-service
cargo build
cargo install --path . --force
```

## quick start

The simplest configuration is available by using command:

`registration-service --config config.yaml`

See [config](./configuration.md) for more details.

## clients

### cli

Registration CLI is cli utility tool which help to interact with registration service without manually constructing requests

See [cli](./cli.md) for more details.

### api

Example:

```
    use registration_service::{
        client::rest::RegistrationRestClient, context::State, request::Request,
    };

    ...

    let payment_skey = Path::new("payment.skey");
    let payment_skey = Path::new("payment.vkey");
    let payment_skey = Path::new("stake.skey");
    let payment_skey = Path::new("stake.vkey");
    let payment_skey = Path::new("vote.skey");
    
    let registration_client =
        RegistrationRestClient::new_with_token(registration_token, registration_address);

    let registration_request = Request {
        payment_skey,
        payment_vkey,
        stake_skey,
        stake_vkey,
        vote_skey,
    };

    let registration_job_id = registration_client.job_new(registration_request).unwrap();

    let wait = WaitBuilder::new().tries(10).sleep_between_tries(10).build();
    println!("waiting for registration job");
    let registration_jobs_status = registration_client
        .wait_for_job_finish(registration_job_id.clone(), wait)
        .unwrap();
    println!("{:?}", registration_jobs_status);

    let qr_code_path = temp_dir.child("qr_code");
    std::fs::create_dir_all(qr_code_path.path()).unwrap();

    let qr_code = registration_client
        .download_qr(registration_job_id.clone(), qr_code_path.path())
        .unwrap();
    let voting_key_sk = registration_client
        .get_catalyst_sk(registration_job_id)
        .unwrap();
```

NOTE: see [cardano cli guide](https://developers.cardano.org/docs/stake-pool-course/handbook/keys-addresses/) for information how to create payment and stake files.
