use rand::RngCore;
use safe_arith::{SafeArith};
use serde_derive::{Deserialize, Serialize};
use ssz::{Decode, DecodeError, Encode};
use std::fmt;
use std::hash::Hash;
use std::iter::Iterator;

#[cfg(feature = "legacy-arith")]
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, Sub, SubAssign};


#[derive(
    arbitrary::Arbitrary,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
)]
#[serde(transparent)]
pub struct Slot(#[serde(with = "serde_utils::quoted_u64")] u64);

#[derive(
    arbitrary::Arbitrary,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
)]
#[serde(transparent)]
pub struct Epoch(#[serde(with = "serde_utils::quoted_u64")] u64);

impl_common!(Slot);
impl_common!(Epoch);

impl Slot {
    pub const fn new(slot: u64) -> Slot {
        Slot(slot)
    }

    pub fn epoch(self, slots_per_epoch: u64) -> Epoch {
        Epoch::new(self.0)
            .safe_div(slots_per_epoch)
            .expect("slots_per_epoch is not 0")
    }

    pub fn max_value() -> Slot {
        Slot(u64::max_value())
    }
}

impl Epoch {
    pub const fn new(slot: u64) -> Epoch {
        Epoch(slot)
    }
}