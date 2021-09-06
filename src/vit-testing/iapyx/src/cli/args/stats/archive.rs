use crate::cli::args::stats::IapyxStatsCommandError;
use crate::stats::archive::{load_from_csv, load_from_folder, ArchiveStats};
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct ArchiveCommand {
    #[structopt(long = "csv", required_unless = "folder")]
    pub csv: Option<PathBuf>,

    #[structopt(long = "folder", required_unless = "csv")]
    pub folder: Option<PathBuf>,

    #[structopt(long = "output")]
    pub output: Option<PathBuf>,

    #[structopt(short = "d", long = "distribution")]
    pub calculate_distribution: bool,

    #[structopt(subcommand)]
    pub command: Command,
}

impl ArchiveCommand {
    pub fn exec(&self) -> Result<(), IapyxStatsCommandError> {
        let archiver: ArchiveStats = {
            if let Some(csv) = &self.csv {
                load_from_csv(&csv)?.into()
            } else if let Some(folder) = &self.folder {
                load_from_folder(&folder)?.into()
            } else {
                panic!("no csv nor folder defined");
            }
        };

        let result = match &self.command {
            Command::VotesByCaster => archiver.number_of_votes_per_caster()?,
            Command::VotesBySlot => archiver.number_of_tx_per_slot()?,
            Command::BatchSizeByCaster(batch_size_by_caster) => {
                batch_size_by_caster.exec(archiver)?
            }
        };

        if self.calculate_distribution {
            self.write_to_file_or_print(ArchiveStats::calculate_distribution(&result)?)?;
        } else {
            self.write_to_file_or_print(result)?;
        }
        Ok(())
    }

    fn write_to_file_or_print<K: serde::Serialize + Ord + Debug, V: serde::Serialize + Debug>(
        &self,
        result: BTreeMap<K, V>,
    ) -> Result<(), std::io::Error> {
        if let Some(output) = &self.output {
            write_to_csv(output, result)?;
        } else {
            println!("{:?}", result);
        }
        Ok(())
    }
}

pub fn write_to_csv<P: AsRef<Path>, K: serde::Serialize + Ord, V: serde::Serialize>(
    output: P,
    result: BTreeMap<K, V>,
) -> Result<(), std::io::Error> {
    let mut writer = {
        let path = output.as_ref().to_path_buf();
        let file = std::fs::File::create(path).unwrap();
        csv::Writer::from_writer(file)
    };

    for (key, value) in result {
        writer.serialize(&(key, value))?;
    }
    Ok(())
}

#[derive(StructOpt, Debug)]
pub enum Command {
    VotesByCaster,
    VotesBySlot,
    BatchSizeByCaster(BatchSizeByCaster),
}

#[derive(StructOpt, Debug)]
pub struct BatchSizeByCaster {
    #[structopt(short = "s", long = "slots-in-epoch")]
    pub slots_in_epoch: u32,
}

impl BatchSizeByCaster {
    pub fn exec(
        &self,
        archiver: ArchiveStats,
    ) -> Result<BTreeMap<String, usize>, IapyxStatsCommandError> {
        Ok(archiver.batch_size_per_caster(self.slots_in_epoch)?)
    }
}
