mod account;
mod post;
mod settings;

pub use account::AccountRequestGen;
pub use post::{BatchWalletRequestGen, RequestGenError, WalletRequestGen};
pub use settings::SettingsRequestGen;
