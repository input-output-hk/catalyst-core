use catalyst_toolbox::utils;
use catalyst_toolbox::vca_reviews::{read_vca_reviews_aggregated_file, Error as ReviewsError};

use jcli_lib::utils::io::open_file_write;
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Review(#[from] ReviewsError),

    #[error("Error while serializing reviews to json")]
    SerializeToJson(#[from] serde_json::Error),

    #[error("Error while serializing reviews to csv")]
    SerializeToCsv(#[from] csv::Error),

    #[error("Invalid output format {0}. Only 'csv' and 'json' are supported")]
    InvalidFormat(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Debug)]
pub enum OutputFormat {
    Csv,
    Json,
}

#[derive(StructOpt)]
pub enum Reviews {
    Export(Export),
}

#[derive(StructOpt)]
pub struct Export {
    /// Path to vca aggregated file
    #[structopt(long)]
    from: PathBuf,
    /// Output file
    #[structopt(long)]
    to: PathBuf,
    /// Output format either csv or json
    #[structopt(long, default_value = "csv")]
    format: OutputFormat,
}

impl FromStr for OutputFormat {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "csv" => Ok(Self::Csv),
            "json" => Ok(Self::Json),
            other => Err(Error::InvalidFormat(other.to_string())),
        }
    }
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Reviews {
    pub fn exec(self) -> Result<(), Error> {
        match self {
            Reviews::Export(transform) => transform.exec()?,
        };
        Ok(())
    }
}

impl Export {
    pub fn exec(self) -> Result<(), Error> {
        let Self { from, to, format } = self;

        let reviews = read_vca_reviews_aggregated_file(&from)?;
        match format {
            OutputFormat::Csv => {
                utils::csv::dump_data_to_csv(reviews.iter(), &to)?;
            }
            OutputFormat::Json => {
                serde_json::to_writer(open_file_write(&Some(to))?, &reviews)?;
            }
        };
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::{Export, OutputFormat};
    use catalyst_toolbox::utils::csv;
    use vit_servicing_station_lib::db::models::community_advisors_reviews::AdvisorReview;

    #[test]
    fn test_output_csv() {
        let resource_input = "./resources/testing/valid_assessments.csv";
        let tmp_file = assert_fs::NamedTempFile::new("outfile.csv").unwrap();

        let export = Export {
            from: resource_input.into(),
            to: tmp_file.path().into(),
            format: OutputFormat::Csv,
        };

        export.exec().unwrap();
        let reviews: Vec<AdvisorReview> = csv::load_data_from_csv::<_, b','>(&tmp_file).unwrap();
        assert_eq!(reviews.len(), 1);
    }
}
