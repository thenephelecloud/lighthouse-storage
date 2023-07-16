use ethereum_types::{H160};
#[macro_use]
pub mod slot_epoch_macros;
pub mod eth_spec;
pub mod slot_epoch;
pub mod nephele_spec;
//pub use crate::eth_spec::EthSpecId;
//pub use crate::slot_epoch::{Epoch, Slot};
pub type Address = H160;