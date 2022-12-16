use cardano_serialization_lib::metadata::MetadataJsonSchema;
use mainnet_lib::{
    CatalystBlockFrostApi, CatalystBlockFrostApiError, GeneralTransactionMetadataInfo,
};
use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<(), CatalystBlockFrostApiError> {
    let params = Command::from_args();

    let api = CatalystBlockFrostApi::new()?;
    let registrations = api.list_registrations_for_address(&params.address).await?;

    registrations.iter().for_each(|reg| {
        println!(
            "{}",
            reg.to_json_string(MetadataJsonSchema::BasicConversions)
                .unwrap()
        )
    });
    Ok(())
}

#[derive(StructOpt, Debug)]
pub struct Command {
    #[structopt(env = "ADDRESS")]
    pub address: String,
}
