//! This crate primarily exists to serve the main storage crate, triggering proof generation 
//! based off changes in the content directory. Janitor is not intended to interact with the protocol,
//! and serves as a low effort watchdog for a simple single node CDN. In the future, as the protocol is built 
//! with backend agnosticism in mind, janitor will be replaced with a more robust and protocol aware backend.

use crate::consensus::Eth2Config;
use storage::StorageMeta;

pub mod watchdog;
