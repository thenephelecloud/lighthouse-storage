//! This crate primarily exists to serve the `common/eth2_network_configs` crate, by providing the
//! canonical list of built-in-networks and some tooling to help include those configurations in the
//! `lighthouse` binary.
//!
//! It also provides some additional structs which are useful to other components of `lighthouse`
//! (e.g., `Eth2Config`).

use types::Eth2Config;
use storage::StorageMeta;

/// The "extended" core configuration of a Lighthouse beacon node.
/// The extension is limited to including a storagemeta struct.
#[derive(Debug, Clone)]
pub struct ExtConfig {
    pub eth2_config: Eth2Config,
    pub storage_meta: StorageMeta,
}

impl Default for ExtConfig {
    fn default() -> Self {
            eth2_config: Eth2Config::default(),
            storage_meta: StorageMeta::default(),
        }
    }
}

impl ExtConfig {
    pub fn mainnet() -> Self {
        Self {
            eth2_config: Eth2Config::mainnet(),
            storage_meta: StorageMeta::mainnet(),
        }
    }

    pub fn minimal() -> Self {
        Self {
            eth2_config: Eth2Config::minimal(),
            storage_meta: StorageMeta::minimal(),
        }
    }

    pub fn gnosis() -> Self {
        Self {
            eth2_config: Eth2Config::gnosis(),
            storage_meta: StorageMeta::testnet(),
        }
    }
}