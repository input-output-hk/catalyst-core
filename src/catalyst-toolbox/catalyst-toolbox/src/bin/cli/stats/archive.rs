use catalyst_toolbox::{
    stats::archive::{load_from_csv, load_from_folder, ArchiveStats},
    utils::csv::dump_to_csv_or_print,
};
use color_eyre::Report;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::path::PathBuf;
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
    pub fn exec(self) -> Result<(), Report> {
        let archiver: ArchiveStats = {
            if let Some(csv) = &self.csv {
                load_from_csv(csv)?.into()
            } else if let Some(folder) = &self.folder {
                load_from_folder(folder)?.into()
            } else {
                panic!("no csv nor folder defined");
            }
        };

        match &self.command {
            Command::VotesByCaster => {
                let result = archiver.number_of_votes_per_caster();
                if self.calculate_distribution {
                    let dist = ArchiveStats::calculate_distribution(&result);
                    dump_to_csv_or_print(self.output, dist.values())?;
                } else {
                    dump_to_csv_or_print(self.output, result.values())?;
                }
            }
            Command::VotesBySlot => {
                let result = archiver.number_of_tx_per_slot();
                if self.calculate_distribution {
                    let dist = ArchiveStats::calculate_distribution(&result);
                    dump_to_csv_or_print(self.output, dist.values())?;
                } else {
                    dump_to_csv_or_print(self.output, result.values())?;
                }
            }
            Command::BatchSizeByCaster(batch_size_by_caster) => {
                let result = batch_size_by_caster.exec(archiver)?;
                if self.calculate_distribution {
                    let dist = ArchiveStats::calculate_distribution(&result);
                    dump_to_csv_or_print(self.output, dist.values())?;
                } else {
                    dump_to_csv_or_print(self.output, result.values())?;
                }
            }
        };

        Ok(())
    }
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
    pub fn exec(&self, archiver: ArchiveStats) -> Result<BTreeMap<String, usize>, Report> {
        Ok(archiver.max_batch_size_per_caster(self.slots_in_epoch)?)
    }
}
