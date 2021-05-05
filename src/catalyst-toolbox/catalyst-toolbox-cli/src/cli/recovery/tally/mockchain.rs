use chain_impl_mockchain::block::HeaderId;
use chain_impl_mockchain::fee::LinearFee;
use chain_impl_mockchain::fragment::Fragment;
use chain_impl_mockchain::ledger::{Error, Ledger, LedgerParameters};
use chain_impl_mockchain::rewards::TaxType;
use jormungandr_lib::interfaces::PersistentFragmentLog;

use std::collections::vec_deque::VecDeque;
use std::iter::FromIterator;

pub fn generate_ledger_from_fragments<'block0>(
    block0_header_id: HeaderId,
    block0_fragments: impl Iterator<Item = &'block0 Fragment>,
    logged_fragments: impl Iterator<Item = PersistentFragmentLog>,
) -> Result<(Ledger, VecDeque<PersistentFragmentLog>), chain_impl_mockchain::ledger::Error> {
    let mut ledger = Ledger::new(block0_header_id, block0_fragments)?;
    let parameters = ledger.get_ledger_parameters();
    let block_date = ledger.date();
    let mut fragments = VecDeque::from_iter(logged_fragments);
    println!("{}", fragments.len());
    let mut counter = 0;
    let mut tally = None;
    while !fragments.is_empty() && counter < fragments.len() {
        counter += 1;
        match fragments.pop_front() {
            None => {
                break;
            }
            Some(fragment_log) => {
                if matches!(&fragment_log.fragment, Fragment::VoteTally(_)) {
                    tally = Some(fragment_log.fragment.clone());
                    continue;
                }
                match ledger.apply_fragment(&parameters, &fragment_log.fragment, block_date) {
                    Ok(_) => {
                        counter = 0;
                    }
                    Err(e) => {
                        dbg!(e);
                        fragments.push_back(fragment_log);
                    }
                }
            }
        }
    }
    if let Some(tally_fragment) = tally {
        ledger.apply_fragment(&parameters, &tally_fragment, block_date)?;
    }
    Ok((ledger, fragments))
}

#[cfg(test)]
mod test {
    use crate::cli::recovery::tally::fragments::load_fragments_from_folder_path;
    use crate::cli::recovery::tally::mockchain::generate_ledger_from_fragments;
    use chain_impl_mockchain::block::Block;
    use chain_ser::deser::Deserialize;
    use std::io::BufReader;
    use std::path::PathBuf;

    fn read_block0(path: PathBuf) -> std::io::Result<Block> {
        let reader = std::fs::File::open(path)?;
        Ok(Block::deserialize(BufReader::new(reader)).unwrap())
    }

    #[test]
    fn test_vote_flow() -> std::io::Result<()> {
        let path: PathBuf = r"D:\projects\rust\catalyst-toolbox\vote_flow_testing\fragments_log"
            .parse()
            .unwrap();

        let fragments = load_fragments_from_folder_path(&path)?;

        let block0 =
            read_block0(r"D:\projects\rust\catalyst-toolbox\vote_flow_testing\block-0.bin".into())?;
        dbg!(&block0);

        let headerId = block0.header.hash();
        let initial_fragments = block0.fragments();

        let (ledger, unprocessed) =
            generate_ledger_from_fragments(headerId, initial_fragments, fragments).unwrap();
        println!("{}", unprocessed.len());
        Ok(())
    }
}
