pub(crate) const ADDR_BYTES: usize = (usize::BITS / 8) as usize;

pub(crate) mod bcst;
pub(crate) mod data;
pub(crate) mod diff;
#[allow(static_mut_refs)]
pub(crate) mod languages;
pub(crate) mod linear;
pub(crate) mod linguist;
pub(crate) mod merge;
pub(crate) mod metadata;
pub(crate) mod patch;
pub(crate) mod rcst;
pub(crate) mod serde;
