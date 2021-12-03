use crate::cli::args::stats::snapshot::Command as SnapshotCommand;
use crate::cli::args::stats::IapyxStatsCommandError;
use crate::stats::archive::{load_from_csv, load_from_folder, ArchiveStats};
use crate::stats::block0::wallets::calculate_wallet_distribution_from_initials_utxo;
use crate::stats::snapshot::read_initials_from_file;
use jormungandr_lib::interfaces::Initial;
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

        match &self.command {
            Command::VotesByCaster => {
                let result = archiver.number_of_votes_per_caster()?;
                if self.calculate_distribution {
                    self.write_to_file_or_print(ArchiveStats::calculate_distribution(&result)?)?;
                } else {
                    self.write_to_file_or_print(result)?;
                }
            }
            Command::VotesBySlot => {
                let result = archiver.number_of_tx_per_slot()?;
                if self.calculate_distribution {
                    self.write_to_file_or_print(ArchiveStats::calculate_distribution(&result)?)?;
                } else {
                    self.write_to_file_or_print(result)?;
                }
            }
            Command::BatchSizeByCaster(batch_size_by_caster) => {
                let result = batch_size_by_caster.exec(archiver)?;
                if self.calculate_distribution {
                    self.write_to_file_or_print(ArchiveStats::calculate_distribution(&result)?)?;
                } else {
                    self.write_to_file_or_print(result)?;
                }
            }
            Command::ActiveVoters(active_voters) => {
                active_voters.exec(archiver)?;
            }
        };

        Ok(())
    }

    pub fn calculate_distribution() {}

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
    ActiveVoters(ActiveVoters),
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

#[derive(StructOpt, Debug)]
pub struct ActiveVoters {
    #[structopt(short = "s", long = "snapshot")]
    pub snapshot: PathBuf,

    #[structopt(long = "support-lovelace")]
    pub support_lovelace: bool,

    #[structopt(long = "threshold")]
    pub threshold: u64,

    #[structopt(subcommand)]
    pub command: SnapshotCommand,
}

impl ActiveVoters {
    pub fn exec(&self, archiver: ArchiveStats) -> Result<(), IapyxStatsCommandError> {
        println!("calculating active voters..");
        let voters = archiver.distinct_casters()?;

        let mut initials = vec![];
        for entry in read_initials_from_file(&self.snapshot)? {
            if let Initial::Fund(snapshot_initials) = entry {
                for initial_utxo in snapshot_initials {
                    if voters.contains(&initial_utxo.address.to_string()) {
                        initials.push(initial_utxo);
                    }
                }
            }
        }

        match self.command {
            SnapshotCommand::Count => calculate_wallet_distribution_from_initials_utxo(
                initials,
                vec![],
                self.threshold,
                self.support_lovelace,
            )?
            .print_count_per_level(),
            SnapshotCommand::Ada => calculate_wallet_distribution_from_initials_utxo(
                initials,
                vec![],
                self.threshold,
                self.support_lovelace,
            )?
            .print_ada_per_level(),
        };

        Ok(())
    }
}
