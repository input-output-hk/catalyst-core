/// the default strategies, in order of importance, to apply
/// when selecting inputs and outputs of a transaction
pub const DEFAULT_STRATEGIES: &[Strategy] = &[
    StrategyBuilder::most_private().build(),
    StrategyBuilder::most_efficient().build(),
];

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum InputStrategy {
    /// try to get the most optimise transaction consuming the
    /// wallet's utxos in the most efficient way.
    ///
    BestEffort,

    /// preserve the privacy of the UTxOs
    ///
    /// This means the transaction will be only composed of
    /// inputs of the same public key. If a change needs created
    /// it will create it to a different unused change, this may
    /// create dust
    ///
    /// This option is incompatible with the `UTXO_CHANGE_TO_ACCOUNT`
    PrivacyPreserving,
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum OutputStrategy {
    /// If the transaction needs to offload extra inputs in an
    /// extra change output then chose the best solution with the
    /// given circumstances
    ///
    /// if there are only single utxos as input the change will be
    /// offloaded to a new change output with a new utxo.
    ///
    /// if there's group inputs for the same key, the group account
    /// will be used to offload the change (order may matter, i.e.
    /// if there's multiple inputs with different account the first
    /// account will be used).
    BestEffort,

    /// Along with privacy preserving, this one will have the interesting
    /// property to redistribute the change into multiple distinct utxos
    ///
    /// however, if the change is only too small (less than 10x dust like):
    ///
    /// * if one of the inputs contain a group key, the change will be distributed
    ///   to the group account
    /// * if there's no account, the change will go to a new utxo
    UtxoReshuffle,
}

pub struct Strategy {
    input: InputStrategy,
    output: OutputStrategy,
}

pub struct StrategyBuilder {
    input: InputStrategy,
    output: OutputStrategy,
}

impl Strategy {
    pub fn input(&self) -> InputStrategy {
        self.input
    }

    pub fn output(&self) -> OutputStrategy {
        self.output
    }
}

impl StrategyBuilder {
    pub const fn most_private() -> Self {
        Self {
            input: InputStrategy::PrivacyPreserving,
            output: OutputStrategy::UtxoReshuffle,
        }
    }

    pub const fn most_efficient() -> Self {
        Self {
            input: InputStrategy::BestEffort,
            output: OutputStrategy::BestEffort,
        }
    }

    pub const fn build(&self) -> Strategy {
        Strategy {
            input: self.input,
            output: self.output,
        }
    }
}

impl Default for StrategyBuilder {
    fn default() -> Self {
        Self {
            input: InputStrategy::BestEffort,
            output: OutputStrategy::BestEffort,
        }
    }
}
