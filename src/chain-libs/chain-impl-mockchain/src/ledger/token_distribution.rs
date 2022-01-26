use crate::{account, ledger::Error, tokens::identifier::TokenIdentifier, value::Value};
use imhamt::Hamt;
use std::collections::hash_map::DefaultHasher;

pub struct TokenDistribution<T> {
    token_totals: TokenTotals,
    account_ledger: account::Ledger,
    token: T,
}

impl TokenDistribution<()> {
    pub fn new(token_totals: TokenTotals, account_ledger: account::Ledger) -> Self {
        Self {
            token_totals,
            account_ledger,
            token: (),
        }
    }

    pub fn token(self, token: TokenIdentifier) -> TokenDistribution<TokenIdentifier> {
        TokenDistribution {
            token_totals: self.token_totals,
            account_ledger: self.account_ledger,
            token,
        }
    }
}

impl TokenDistribution<TokenIdentifier> {
    pub fn get_total(&self) -> Value {
        self.token_totals
            .get_total(&self.token)
            // maybe this should be an error? but errors in the tally are not really recoverable
            // OTOH, this will make the tally succeed but not have any effect, so it's more or less
            // the same effect
            .unwrap_or_else(Value::zero)
    }

    pub fn get_account(&self, account: &account::Identifier) -> Option<Value> {
        self.account_ledger
            .get_state(account)
            // It could be argued that this is silently hiding an error, and that's more or
            // less true, since having a vote from an account not in the account ledger would
            // be unexpected. But throwing an error here would abort the whole tally, and there
            // is really no way of fixing things if that's the case.
            //
            // This is mostly theoretical though, since I don't think that can happen, so this
            // `ok` should always return Some.
            .ok()
            .and_then(|account_state| account_state.tokens.lookup(&self.token))
            .copied()
    }
}

#[derive(Clone, PartialEq, Eq, Default)]
pub struct TokenTotals(Hamt<DefaultHasher, TokenIdentifier, Value>);

impl TokenTotals {
    #[must_use = "Does not modify the internal state"]
    pub fn add(&self, token: TokenIdentifier, value: Value) -> Result<TokenTotals, Error> {
        self.0
            .insert_or_update(token, value, |v| v.checked_add(value).map(Some))
            .map(TokenTotals)
            .map_err(Into::into)
    }

    pub fn get_total(&self, token: &TokenIdentifier) -> Option<Value> {
        self.0.lookup(token).copied()
    }
}
