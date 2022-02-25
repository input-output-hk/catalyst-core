use crate::{
    account::{self, LedgerError},
    ledger::Error,
    tokens::identifier::TokenIdentifier,
    value::Value,
};
use imhamt::Hamt;
use std::collections::hash_map::DefaultHasher;

#[derive(PartialEq, Eq)]
pub struct TokenDistribution<'a, T: Clone + PartialEq + Eq> {
    token_totals: &'a TokenTotals,
    account_ledger: &'a account::Ledger,
    token: &'a T,
}

impl Clone for TokenDistribution<'_, ()> {
    fn clone(&self) -> Self {
        Self {
            token_totals: self.token_totals,
            account_ledger: self.account_ledger,
            token: &(),
        }
    }
}

impl<'a> TokenDistribution<'a, ()> {
    pub fn new(token_totals: &'a TokenTotals, account_ledger: &'a account::Ledger) -> Self {
        Self {
            token_totals,
            account_ledger,
            token: &(),
        }
    }

    pub fn token(self, token: &'a TokenIdentifier) -> TokenDistribution<TokenIdentifier> {
        TokenDistribution {
            token_totals: self.token_totals,
            account_ledger: self.account_ledger,
            token,
        }
    }
}

impl<'a> TokenDistribution<'a, TokenIdentifier> {
    pub fn get_total(&self) -> Value {
        self.token_totals
            .get_total(self.token)
            // maybe this should be an error? but errors in the tally are not really recoverable
            // OTOH, this will make the tally succeed but not have any effect, so it's more or less
            // the same effect
            .unwrap_or_else(Value::zero)
    }

    pub fn get_account(&self, account: &account::Identifier) -> Result<Option<Value>, LedgerError> {
        self.account_ledger
            .get_state(account)
            .map(|account_state| account_state.tokens.lookup(self.token).copied())
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
