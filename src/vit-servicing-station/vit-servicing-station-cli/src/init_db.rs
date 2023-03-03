use crate::task::ExecTask;
use clap::Parser;
use thiserror::Error;
use vit_servicing_station_lib::db::{
    load_db_connection_pool, /*migrations::initialize_db_with_migration,*/ Error as DbPoolError,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error connecting db pool: {0}")]
    DbPool(#[from] DbPoolError),
    #[error("Error connecting to db: {0}")]
    DbConnection(#[from] r2d2::Error),
    //#[error("Error running db migrations: {0}")]
    //InitializeDbWithMigration(
    //    #[from] vit_servicing_station_lib::db::migrations::InitializeDbWithMigrationError,
    //),
}

#[derive(Debug, PartialEq, Eq, Parser)]
pub enum Db {
    /// Initialize a DB with the proper migrations, DB file is created if not exists.
    Init {
        /// URL of the vit-servicing-station database to interact with
        #[clap(long = "db-url")]
        db_url: String,
    },
}

impl Db {
    fn init_with_migrations(db_url: &str) -> Result<(), Error> {
        let pool = load_db_connection_pool(db_url)?;
        let _db_conn = pool.get()?;
        //initialize_db_with_migration(&db_conn)?;
        Ok(())
    }
}

impl ExecTask for Db {
    type ResultValue = ();
    type Error = Error;
    fn exec(&self) -> Result<Self::ResultValue, Error> {
        match self {
            Db::Init { db_url } => Db::init_with_migrations(db_url),
        }
    }
}
