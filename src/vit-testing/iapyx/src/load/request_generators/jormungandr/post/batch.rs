use crate::load::{MultiController, MultiControllerError};
use crate::utils::expiry;
use crate::Wallet;
use jormungandr_automation::testing::VoteCastCounter;
use jortestkit::load::{Id, Request, RequestFailure, RequestGenerator};
use rand::RngCore;
use rand_core::OsRng;
use std::collections::HashSet;
use std::time::Instant;
use thor::BlockDateGenerator;
use valgrind::SettingsExtensions;
use vit_servicing_station_lib::db::models::proposals::FullProposalInfo;
use wallet::Settings;
use wallet_core::Choice;

pub struct BatchWalletRequestGen {
    rand: OsRng,
    batch_size: usize,
    multi_controller: MultiController,
    proposals: Vec<FullProposalInfo>,
    options: Vec<u8>,
    use_v1: bool,
    wallet_index: usize,
    update_account_before_vote: bool,
    vote_cast_counter: VoteCastCounter,
    block_date_generator: BlockDateGenerator,
    settings: Settings,
}

impl BatchWalletRequestGen {
    pub fn new(
        multi_controller: MultiController,
        batch_size: usize,
        use_v1: bool,
        update_account_before_vote: bool,
        voting_group: &str,
    ) -> Result<Self, MultiControllerError> {
        let proposals = multi_controller.proposals(voting_group)?;
        let voting_groups_vote_plans_ids: HashSet<String> = proposals
            .iter()
            .map(|p| p.voteplan.chain_voteplan_id.to_string())
            .collect();

        println!("direct vote plans: {voting_groups_vote_plans_ids:?}");

        let vote_plans = multi_controller.backend().vote_plan_statuses()?;
        let settings = multi_controller.backend().settings()?;

        let options = proposals[0]
            .proposal
            .chain_vote_options
            .0
            .values()
            .copied()
            .collect();

        let mut vote_plans_registry = Vec::new();

        for v in vote_plans
            .iter()
            .filter(|v| voting_groups_vote_plans_ids.contains(&v.id.to_string()))
        {
            vote_plans_registry.push((
                v.id.into(),
                proposals
                    .len()
                    .try_into()
                    .map_err(|_| MultiControllerError::InvalidProposalsLen(v.id))?,
            ));
        }

        let vote_cast_counter =
            VoteCastCounter::new(multi_controller.wallet_count(), vote_plans_registry);

        Ok(Self {
            batch_size,
            use_v1,
            multi_controller,
            rand: OsRng,
            proposals,
            options,
            wallet_index: 0,
            update_account_before_vote,
            vote_cast_counter,
            settings: settings.clone().into_wallet_settings(),
            block_date_generator: expiry::default_block_date_generator(&settings),
        })
    }

    pub fn next_usize(&mut self) -> usize {
        self.rand.next_u32() as usize
    }

    pub fn random_votes(&mut self) -> Result<Vec<Option<Id>>, MultiControllerError> {
        let wallet_index = {
            self.wallet_index += 1;
            if self.wallet_index >= self.multi_controller.wallet_count() {
                self.wallet_index = 0;
            }
            self.wallet_index
        };

        // update state of wallet only before first vote.
        // Then rely on mechanism of spending counter auto-update
        if self.update_account_before_vote {
            self.multi_controller
                .update_wallet_state_if(wallet_index, &|wallet: &Wallet| {
                    wallet.spending_counter()[0] == 0
                })?;
        }

        let batch_size = self.batch_size;
        let options = self.options.clone();

        let counter = self
            .vote_cast_counter
            .advance_batch(batch_size, wallet_index)
            .unwrap();

        let mut proposals = Vec::new();

        for item in &counter {
            for i in item.range() {
                match self.proposals.iter().find(|x| {
                    x.voteplan.chain_voteplan_id == item.id().to_string()
                        && (x.voteplan.chain_proposal_index % i64::from(u8::MAX)) == i as i64
                }) {
                    Some(proposal) => {
                        println!("vote on: {}/{}", proposal.voteplan.chain_voteplan_id, i);
                        proposals.push(proposal.clone());
                    }
                    None => {
                        return Err(MultiControllerError::NotEnoughProposals);
                    }
                }
            }
        }

        let choices: Vec<Choice> =
            std::iter::from_fn(|| Some(self.next_usize() % self.options.len()))
                .take(batch_size)
                .map(|index| Choice::new(*options.get(index).unwrap()))
                .collect();

        self.multi_controller
            .votes_batch(
                wallet_index,
                self.use_v1,
                proposals.iter().zip(choices).collect(),
                &self.block_date_generator.block_date(),
            )
            .map(|x| {
                x.into_iter()
                    .map(|s| Some(s.to_string()))
                    .collect::<Vec<Option<Id>>>()
            })
    }
}

impl RequestGenerator for BatchWalletRequestGen {
    fn split(mut self) -> (Self, Option<Self>) {
        let wallets_len = self.multi_controller.wallets.len();
        if wallets_len <= 1 {
            return (self, None);
        }
        let wallets = self.multi_controller.wallets.split_off(wallets_len / 2);
        let new_gen = Self {
            rand: self.rand,
            multi_controller: MultiController {
                wallets,
                backend: self.multi_controller.backend.clone(),
                settings: self.multi_controller.settings.clone(),
            },
            proposals: self.proposals.clone(),
            options: self.options.clone(),
            use_v1: self.use_v1,
            batch_size: self.batch_size,
            wallet_index: 0,
            update_account_before_vote: self.update_account_before_vote,
            vote_cast_counter: self.vote_cast_counter.clone(),
            settings: self.settings.clone(),
            block_date_generator: self.block_date_generator.clone(),
        };

        (self, Some(new_gen))
    }

    fn next(&mut self) -> Result<Request, RequestFailure> {
        let start = Instant::now();
        match self.random_votes() {
            Ok(ids) => Ok(Request {
                ids,
                duration: start.elapsed(),
            }),
            Err(e) => Err(RequestFailure::General(format!("{e:?}"))),
        }
    }
}
