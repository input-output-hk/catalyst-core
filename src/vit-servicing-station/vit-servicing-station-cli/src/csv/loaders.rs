use crate::{db_utils::db_file_exists, task::ExecTask};
use csv::Trim;
use serde::de::DeserializeOwned;
use std::io;
use structopt::StructOpt;
use vit_servicing_station_lib::db::{
    load_db_connection_pool, models::funds::Fund, models::proposals::Proposal,
    models::voteplans::Voteplan,
};

#[derive(Debug, PartialEq, StructOpt)]
pub enum CSVDataCmd {
    /// Add provided tokens to database. If --tokens is not provided the binary will read them from the `stdin`
    Dump {
        /// URL of the vit-servicing-station database to interact with
        #[structopt(long = "db-url")]
        db_url: String,

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
    fn load_from_csv<T: DeserializeOwned>(csv_path: &str) -> io::Result<Vec<T>> {
        let mut reader = csv::ReaderBuilder::new()
            .flexible(true)
            .has_headers(true)
            .quoting(true)
            .quote(b'"')
            .trim(Trim::All)
            .from_path(csv_path)?;
        let mut results = Vec::new();
        for record in reader.deserialize() {
            results.push(record?);
        }
        Ok(results)
    }

    fn handle_dump(
        db_url: &str,
        funds_path: &str,
        voteplans_path: &str,
        proposals_path: &str,
    ) -> io::Result<()> {
        db_file_exists(db_url)?;
        let funds = CSVDataCmd::load_from_csv::<Fund>(funds_path)?;
        if funds.len() != 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Wrong number of input fund, just one fund data can be process at a time",
            ));
        }
        let mut voteplans = CSVDataCmd::load_from_csv::<Voteplan>(voteplans_path)?;
        let mut proposals: Vec<Proposal> =
            CSVDataCmd::load_from_csv::<super::models::Proposal>(proposals_path)?
                .iter()
                .cloned()
                .map(Into::into)
                .collect();
        // start db connection
        let pool = load_db_connection_pool(db_url)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, format!("{}", e)))?;
        let db_conn = pool
            .get()
            .map_err(|e| io::Error::new(io::ErrorKind::NotConnected, format!("{}", e)))?;

        // insert fund and retrieve fund with id
        let fund =
            vit_servicing_station_lib::db::queries::funds::insert_fund(funds[0].clone(), &db_conn)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{}", e)))?;

        // apply fund id in voteplans
        for voteplan in voteplans.iter_mut() {
            voteplan.fund_id = fund.id;
        }

        // apply fund id in proposals
        for proposal in proposals.iter_mut() {
            proposal.fund_id = fund.id;
        }

        vit_servicing_station_lib::db::queries::voteplans::batch_insert_voteplans(
            &voteplans, &db_conn,
        )
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{}", e)))?;

        vit_servicing_station_lib::db::queries::proposals::batch_insert_proposals(
            &proposals, &db_conn,
        )
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{}", e)))?;

        Ok(())
    }
}

impl ExecTask for CSVDataCmd {
    type ResultValue = ();

    fn exec(&self) -> std::io::Result<()> {
        match self {
            CSVDataCmd::Dump {
                db_url,
                funds,
                voteplans,
                proposals,
            } => Self::handle_dump(db_url, funds, voteplans, proposals),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
}
