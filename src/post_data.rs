use crate::test_utils::TestRandom;
use crate::{Checkpoint, Hash256, SignedRoot, Slot};

use crate::slot_data::SlotData;
use serde_derive::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash_derive::TreeHash;

/// The data upon which a storage attestation is based.
///
/// Spec *draft*, same as vanilla eth2 attestations
#[derive(
    arbitrary::Arbitrary,
    Debug,
    Clone,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    Hash,
    Encode,
    Decode,
    TreeHash,
    TestRandom,
    Default,
)]
pub struct PoSTData {
    pub proof: Hash256,
}

impl SignedRoot for PoSTData {}

#[cfg(test)]
mod tests {
    use super::*;

    ssz_and_tree_hash_tests!(PoSTData);
}