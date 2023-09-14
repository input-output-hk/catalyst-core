use poem_openapi::{types::Example, NewType};
use serde::Deserialize;

#[derive(NewType, Deserialize)]
#[oai(example = true)]
pub struct StakePublicKey(pub String);

impl Example for StakePublicKey {
    fn example() -> Self {
        Self("ad4b948699193634a39dd56f779a2951a24779ad52aa7916f6912b8ec4702cee".into())
    }
}
