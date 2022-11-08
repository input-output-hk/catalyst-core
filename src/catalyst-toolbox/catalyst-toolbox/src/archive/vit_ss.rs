use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Storage(#[from] chain_storage::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Csv(#[from] csv::Error),

    #[error("Only accounts inputs are supported not Utxos")]
    UnhandledInput,
}

pub fn generate_archive_files(vit_ss_database: &Path, output_dir: &Path) -> Result<(), Error> {
    Ok(())
}
