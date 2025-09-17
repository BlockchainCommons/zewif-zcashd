use zewif::mod_use;

pub mod error;
pub use error::{Error, OptionExt, Result, ResultExt};

mod_use!(bdb_dump);
mod_use!(zcashd_dump);
mod_use!(zcashd_parser);

pub mod migrate;
pub mod parser;
pub mod zcashd_wallet;
pub use migrate::migrate_to_zewif;
pub use zcashd_wallet::ZcashdWallet;
