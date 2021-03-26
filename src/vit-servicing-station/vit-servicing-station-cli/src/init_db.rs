use crate::task::ExecTask;
use std::io;
use structopt::StructOpt;
use vit_servicing_station_lib::db::{
    load_db_connection_pool, migrations::initialize_db_with_migration,
};

#[derive(Debug, PartialEq, StructOpt)]
pub enum Db {
    /// Initialize a DB with the proper migrations, DB file is created if not exists.
    Init {
        /// URL of the vit-servicing-station database to interact with
        #[structopt(long = "db-url")]
        db_url: String,
    },
}

impl Db {
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

impl ExecTask for Db {
    type ResultValue = ();

    fn exec(&self) -> io::Result<Self::ResultValue> {
        match self {
            Db::Init { db_url } => Db::init_with_migrations(db_url),
        }
    }
}
