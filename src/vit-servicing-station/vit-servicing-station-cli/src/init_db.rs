use crate::db_utils::db_file_exists;
use crate::task::ExecTask;
use std::io;
use structopt::StructOpt;
use vit_servicing_station_lib::db::{
    load_db_connection_pool, migrations::initialize_db_with_migration,
};

#[derive(Debug, PartialEq, StructOpt)]
pub enum DB {
    /// Add provided tokens to database. If --tokens is not provided the binary will read them from the `stdin`
    Init {
        /// URL of the vit-servicing-station database to interact with
        #[structopt(long = "db-url")]
        db_url: String,
    },
}

impl DB {
    fn init_with_migrations(db_url: &str) -> io::Result<()> {
        let pool = load_db_connection_pool(db_url)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, format!("{}", e)))?;
        let db_conn = pool
            .get()
            .map_err(|e| io::Error::new(io::ErrorKind::NotConnected, format!("{}", e)))?;
        initialize_db_with_migration(&db_conn);
        Ok(())
    }
}

impl ExecTask for DB {
    type ResultValue = ();

    fn exec(&self) -> io::Result<Self::ResultValue> {
        match self {
            DB::Init { db_url } => DB::init_with_migrations(db_url),
        }
    }
}
