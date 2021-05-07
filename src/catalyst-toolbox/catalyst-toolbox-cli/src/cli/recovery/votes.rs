use super::Error;
use chain_crypto::bech32::Bech32;
use chain_crypto::PublicKey;
use chain_impl_mockchain::fragment::Fragment;
use chain_impl_mockchain::transaction::InputEnum;
use chain_impl_mockchain::vote::Payload;
use jormungandr_lib::interfaces::{
    load_persistent_fragments_logs_from_folder_path, PersistentFragmentLog,
};
use srde::Serialize;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct VotesPrintout {
    /// Path to the block0 binary file
    #[structopt(long)]
    block0_path: PathBuf,

    /// Path to the folder containing the log files used for the tally reconstruction
    #[structopt(long)]
    logs_path: PathBuf,

    #[structopt(flatten)]
    output: OutputFile,

    #[structopt(flatten)]
    output_format: OutputFormat,
}

#[derive(Serialize)]
struct VoteCast {
    #[serde(serialize_with = "PublicKey::to_bech32_str")]
    public_key: PublicKey,
    voteplan: VotePlanId,
    proposal: u8,
    choice: u8,
}

impl From<Transaction<VoteCast>> for VoteCast {
    fn from(transaction: Transaction<VoteCast>) -> Self {
        let (vote_cast, identifier, _) = deconstruct_transaction(transaction);
        let choice = if let Payload::Public { choice } = vote_cast.payload() {
            choice
        } else {
            panic!("cannot handle private votes");
        };
        Self {
            public_key: identifier.into(),
            voteplan: vote_cast.vote_plan(),
            proposal: vote_cast.proposal_index(),
            choice: choice.into(),
        }
    }
}

fn group_by_voter(fragments: impl Iterator<Item = Fragment>) -> HashMap<PublicKey, Vec<VoteCast>> {
    let mut res = HashMap::new();
}

impl VotesPrintout {
    pub fn exec(self) -> Result<(), Error> {
        let reader = std::fs::File::open(path)?;
        Ok(Block::deserialize(BufReader::new(reader)).unwrap());

        let fragments =
            load_persistent_fragments_logs_from_folder_path(&logs_path)?.collect::<Vec<_>>();

        let non_filtered = group_by_voter(fragments);

        let filtered = group_by_voter(filtered_fragments);

        let ledger = Ledger::new(block0.header.id(), block0.fragments()).unwrap();
        let voteplans = ledger.active_vote_plans();
        let (vote_start, vote_end, committee_end) = (
            voteplans[0].vote_start,
            voteplans[0].vote_end,
            voteplans[1].committee_end,
        );

        let mut out_writer = output.open()?;
        let content = output_format.format_json(serde_json::to_value(&voteplan_status)?)?;
        out_writer.write_all(content.as_bytes())?;
        let fragments = load_persistent_fragments_logs_from_folder_path(&self.file)?;
        for fragment in fragments {
            let fragment = fragment?;
            if let PersistentFragmentLog {
                fragment: Fragment::VoteCast(transaction),
                ..
            } = fragment
            {
                let vote_cast = transaction.as_slice().payload().into_payload();
                let account = transaction
                    .as_slice()
                    .inputs()
                    .iter()
                    .next()
                    .unwrap()
                    .to_enum();
                let public_key: PublicKey<_> = if let InputEnum::AccountInput(account, _) = account
                {
                    account.to_single_account().unwrap().into()
                } else {
                    panic!("cannot handle utxo inputs");
                };

                println!(
                    "public_key: {} | voteplan: {} | proposal index: {} |   choice: {:?} ",
                    public_key.to_bech32_str(),
                    vote_cast.vote_plan(),
                    vote_cast.proposal_index(),
                    choice
                );
            }
        }

        Ok(())
    }
}
