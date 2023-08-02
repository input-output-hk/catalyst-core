use crate::load::{MultiController, MultiControllerError};
use crate::utils::expiry;
use crate::Wallet;
use chain_impl_mockchain::fragment::FragmentId;
use jormungandr_automation::testing::VoteCastCounter;
use jortestkit::load::{Request, RequestFailure, RequestGenerator};
use rand::seq::SliceRandom;
use rand_core::OsRng;
use std::time::Instant;
use thor::BlockDateGenerator;
use valgrind::SettingsExtensions;
use vit_servicing_station_lib::db::models::proposals::FullProposalInfo;
use wallet::Settings;
use wallet_core::Choice;

///  Vote request generator. Implements `RequestGenerator` interface which incorporates generator
/// to load testing framework. Responsibility is to keep track of wallets under tests and prepare
/// each time valid vote transaction whenever asked. There are two challenges:
/// - keeping track of spending counter and increase it each time fragment is sent. Current limitation
/// is a lack of recovery scenario when transaction is failed (no resend strategy or spending counter revert)
/// - keeping track of valid proposals to vote. One can vote only once per proposal. Duplicated votes will
/// result in failed transaction which can skew load test results. Therefore, we need to also know which
/// proposals are eligible to vote on.  Having in mind internal structure of vote plan (voteplan can have many proposals)
/// and requirement to send batch of votes may result in different proposals from different voteplan.
pub struct WalletRequestGen {
    rand: OsRng,
    multi_controller: MultiController,
    proposals: Vec<FullProposalInfo>,
    options: Vec<u8>,
    wallet_index: usize,
    update_account_before_vote: bool,
    vote_cast_counter: VoteCastCounter,
    block_date_generator: BlockDateGenerator,
    settings: Settings,
}

impl WalletRequestGen {
    /// Creates new object
    ///
    /// # Errors
    ///
    /// On connectivity with backend issues
    #[allow(clippy::cast_possible_truncation)]
    pub fn new(
        multi_controller: MultiController,
        update_account_before_vote: bool,
        group: &str,
    ) -> Result<Self, super::RequestGenError> {
        let proposals = multi_controller.proposals(group)?;
        let vote_plans = multi_controller.backend().vote_plan_statuses()?;
        let settings = multi_controller.backend().settings()?;
        let options = proposals[0]
            .proposal
            .chain_vote_options
            .0
            .values()
            .copied()
            .collect();

        let vote_cast_counter = VoteCastCounter::new(
            multi_controller.wallet_count(),
            vote_plans
                .iter()
                .map(|v| (v.id.into(), v.proposals.len() as u8))
                .collect(),
        );

        Ok(Self {
            multi_controller,
            proposals,
            options,
            wallet_index: 0,
            update_account_before_vote,
            vote_cast_counter,
            rand: OsRng,
            settings: settings.clone().into_wallet_settings(),
            block_date_generator: expiry::default_block_date_generator(&settings),
        })
    }

    /// Sends vote with random choice on behalf of random wallet. Having in mind account based model
    /// in Jormungandr blockchain we need to carefully select wallet which sends transaction.
    /// We should spread the use of wallets evenly to avoid inconsistency in spending-counter values.
    /// This struct does not control how fast votes should be send so we need to avoid situation that
    /// votes from account A will be send without prior confirmation of last vote sent from account A.
    /// Therefore a simple rolling index is use which traverse from left to right of collection.
    /// There is a silent assumption that collection is big enough to not cause mentioned problem
    /// with spending counter inconsistency.
    ///
    /// # Errors
    ///
    ///
    pub fn random_vote(&mut self) -> Result<FragmentId, MultiControllerError> {
        let index = {
            self.wallet_index += 1;
            if self.wallet_index >= self.multi_controller.wallet_count() {
                self.wallet_index = 0;
            }
            self.wallet_index
        };

        // update state of wallet only before first vote.
        // Then relay on mechanism of spending counter auto-update
        if self.update_account_before_vote {
            self.multi_controller
                .update_wallet_state_if(index, &|wallet: &Wallet| {
                    wallet.spending_counter()[0] == 0
                })?;
        }

        let counter = self.vote_cast_counter.advance_single(index)?;
        let index = usize::from(
            counter
                .first()
                .ok_or(MultiControllerError::NoMoreVotesToVote)?
                .first(),
        );
        let proposal = self
            .proposals
            .get(index)
            .ok_or(MultiControllerError::MissingProposal(index))?;
        let choice = Choice::new(
            *self
                .options
                .choose(&mut self.rand)
                .ok_or(MultiControllerError::RandomChoiceFailed)?,
        );
        self.multi_controller.vote(
            index,
            proposal,
            choice,
            self.block_date_generator.block_date(),
        )
    }
}

impl RequestGenerator for WalletRequestGen {
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
        match self.random_vote() {
            Ok(v) => Ok(Request {
                ids: vec![Some(v.to_string())],
                duration: start.elapsed(),
            }),
            Err(e) => Err(RequestFailure::General(format!("{e:?}"))),
        }
    }
}
