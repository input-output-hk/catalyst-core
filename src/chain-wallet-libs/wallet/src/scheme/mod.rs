pub mod rindex;

use thiserror::Error;

pub enum Error {
    DuplicateUtxo,
}
