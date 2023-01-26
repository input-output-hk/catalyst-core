use cardano_serialization_lib::metadata::MetadataJsonSchema;
use clap::Parser;
use mainnet_lib::{
    CatalystBlockFrostApi, CatalystBlockFrostApiError, GeneralTransactionMetadataInfo,
};

#[tokio::main]
async fn main() -> Result<(), CatalystBlockFrostApiError> {
    let params = Command::parse();

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

#[derive(Parser, Debug)]
pub struct Command {
    #[clap(env = "ADDRESS")]
    pub address: String,
}
