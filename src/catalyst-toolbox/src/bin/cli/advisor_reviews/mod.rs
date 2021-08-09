use catalyst_toolbox::vca_reviews::{
    read_vca_reviews_aggregated_file, Error as ReviewsError, TagsMap,
};
use jcli_lib::utils::io::{open_file_read, open_file_write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use structopt::StructOpt;
use vit_servicing_station_lib::db::models::community_advisors_reviews::AdvisorReview;

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

pub enum OutputFormat {
    Csv,
    Json,
}

#[derive(StructOpt)]
pub enum Reviews {
    Transform(Transform),
}

#[derive(StructOpt)]
pub struct Transform {
    /// Path to vca aggreagted file
    from: PathBuf,
    /// Output file
    to: PathBuf,
    /// Output format either csv or json
    #[structopt(default_value = "csv")]
    format: OutputFormat,
    /// Worksheet name
    worksheet: String,
    /// Tags json file
    tags: Option<PathBuf>,
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

impl Reviews {
    pub fn exec(self) -> Result<(), Error> {
        match self {
            Reviews::Transform(transform) => transform.exec()?,
        };
        Ok(())
    }
}

impl Transform {
    pub fn exec(self) -> Result<(), Error> {
        let Self {
            from,
            to,
            format,
            worksheet,
            tags,
        } = self;
        let tags_map: TagsMap = if let Some(tags) = tags {
            serde_json::from_reader(open_file_read(&Some(tags))?)?
        } else {
            TagsMap::default()
        };

        let reviews = read_vca_reviews_aggregated_file(&from, &worksheet, &tags_map)?;
        match format {
            OutputFormat::Csv => {
                write_csv(&reviews, &to)?;
            }
            OutputFormat::Json => {
                serde_json::to_writer(open_file_write(&Some(to))?, &reviews)?;
            }
        };
        Ok(())
    }
}

pub fn write_csv(reviews: &[AdvisorReview], filepath: &Path) -> Result<(), Error> {
    let mut writer = csv::WriterBuilder::new()
        .delimiter(b',')
        .double_quote(true)
        .has_headers(true)
        .from_path(filepath)?;
    for review in &reviews {
        writer.serialize(review)?;
    }
    Ok(())
}
