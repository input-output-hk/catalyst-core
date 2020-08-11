use crate::task::ExecTask;
use csv::Trim;
use std::io;
use structopt::StructOpt;
use vit_servicing_station_lib::db::{
    load_db_connection_pool, models::funds::Fund, models::proposals::Proposal,
    models::voteplans::Voteplan, DBConnection,
};

#[derive(Debug, PartialEq, StructOpt)]
pub enum CSVDataCmd {
    /// Add provided tokens to database. If --tokens is not provided the binary will read them from the `stdin`
    Dump {
        /// List of tokens in URL safe base64. If --tokens is not provided the binary will read them from the `stdin`
        #[structopt(long = "funds")]
        funds: String,

        #[structopt(long = "voteplans")]
        voteplans: String,

        #[structopt(long = "proposals")]
        proposals: String,
    },
}
// type Fundd = (String, String, String, String, i64, i64, i64);
impl CSVDataCmd {
    fn load_funds(csv_path: &str) -> io::Result<Vec<Fund>> {
        let mut reader = csv::ReaderBuilder::new()
            .flexible(true)
            .trim(Trim::All)
            .from_path(csv_path)?;
        for record in reader.deserialize() {
            let record: Fund = record?;
            println!("{:?}", record);
        }
        Ok(Vec::new())
    }

    fn load_voteplans(csv_path: &str) -> Vec<Voteplan> {
        Vec::new()
    }

    fn load_proposals(csv_path: &str) -> Vec<Proposal> {
        Vec::new()
    }
}

impl ExecTask for CSVDataCmd {
    type ResultValue = ();

    fn exec(&self) -> std::io::Result<()> {
        match self {
            CSVDataCmd::Dump {
                funds,
                voteplans,
                proposals,
            } => {
                Self::load_funds(funds)?;
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
}
