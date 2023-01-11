use crate::{
    generators::{BatchFragmentGenerator, FragmentStatusProvider},
    mjolnir_lib::{args::parse_shift, build_monitor, MjolnirError},
};
use chain_addr::Discrimination;
use chain_impl_mockchain::block::BlockDate;
use clap::Parser;
use jormungandr_automation::jormungandr::RemoteJormungandrBuilder;
use jormungandr_lib::crypto::hash::Hash;
use jortestkit::{
    load::ConfigurationBuilder,
    prelude::{parse_progress_bar_mode_from_str, ProgressBarMode},
};
use std::{path::PathBuf, str::FromStr, time::Duration};
use thor::{BlockDateGenerator, DiscriminationExtension, FragmentSenderSetup, Wallet};
#[derive(Parser, Debug)]
pub struct TxOnly {
    /// Number of threads
    #[clap(short = 'c', long = "count", default_value = "3")]
    pub count: usize,

    /// address in format:
    /// /ip4/54.193.75.55/tcp/3000
    #[clap(short = 'e', long = "endpoint")]
    pub endpoint: String,

    /// amount of delay [milliseconds] between sync attempts
    #[clap(long = "delay", default_value = "50")]
    pub delay: u64,

    /// amount of delay [seconds] between sync attempts
    #[clap(short = 'd', long = "duration")]
    pub duration: u64,

    // show progress
    #[clap(
        long = "progress-bar-mode",
        short = 'b',
        default_value = "Monitor",
        value_parser = parse_progress_bar_mode_from_str
    )]
    progress_bar_mode: ProgressBarMode,

    #[clap(short = 'm', long = "measure")]
    pub measure: bool,

    #[clap(long = "key", short = 'k')]
    faucet_key_file: PathBuf,

    #[clap(long = "spending-counter", short = 's')]
    faucet_spending_counter: u32,

    /// Transaction validity deadline (inclusive)
    #[clap(short = 'v', long = "valid-until", conflicts_with = "ttl")]
    valid_until: Option<BlockDate>,

    /// Transaction time to live (can be negative e.g. ~4.2)
    #[clap(short = 't', long= "ttl", default_value = "1.0", value_parser = parse_shift)]
    ttl: (BlockDate, bool),

    /// Set the discrimination type to testing (default is production).
    #[clap(long = "testing")]
    testing: bool,
}

impl TxOnly {
    pub fn exec(&self) -> Result<(), MjolnirError> {
        let mut faucet = Wallet::import_account(
            &self.faucet_key_file,
            Some(self.faucet_spending_counter.into()),
            Discrimination::from_testing_bool(self.testing),
        );
        let remote_jormungandr = RemoteJormungandrBuilder::new("node".to_owned())
            .with_rest(self.endpoint.parse().unwrap())
            .build();

        let settings = remote_jormungandr.rest().settings().unwrap();

        let block0_hash = Hash::from_str(&settings.block0_hash).unwrap();
        let fees = settings.fees.clone();

        let expiry_generator = self
            .valid_until
            .map(BlockDateGenerator::Fixed)
            .unwrap_or_else(|| BlockDateGenerator::rolling(&settings, self.ttl.0, self.ttl.1));

        let mut request_gen = BatchFragmentGenerator::new(
            FragmentSenderSetup::no_verify(),
            remote_jormungandr,
            block0_hash,
            fees,
            expiry_generator,
            10,
        );
        request_gen.fill_from_faucet(&mut faucet);

        let config = ConfigurationBuilder::duration(Duration::from_secs(self.duration))
            .thread_no(self.count)
            .step_delay(Duration::from_millis(self.delay))
            .monitor(build_monitor(&self.progress_bar_mode))
            .shutdown_grace_period(Duration::from_secs(30))
            .build();

        let remote_jormungandr = RemoteJormungandrBuilder::new("node".to_owned())
            .with_rest(self.endpoint.parse().unwrap())
            .build();
        let status_provider = FragmentStatusProvider::new(remote_jormungandr);

        let stats =
            jortestkit::load::start_async(request_gen, status_provider, config, "rest  test");
        if self.measure {
            assert!((stats.calculate_passrate() as u32) > 95);
        }
        Ok(())
    }
}
