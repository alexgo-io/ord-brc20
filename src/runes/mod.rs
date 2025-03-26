use super::*;

pub use {edict::Edict, rune::Rune, rune_id::RuneId, runestone::Runestone};

pub(crate) use {etching::Etching, spaced_rune::SpacedRune};

pub const MAX_DIVISIBILITY: u8 = 38;
#[cfg(test)]
pub(crate) const CLAIM_BIT: u128 = 1 << 48;
pub(crate) const MAX_LIMIT: u128 = 1 << 64;

mod edict;
mod etching;
mod rune;
mod rune_id;
mod runestone;
mod spaced_rune;
pub mod varint;

type Result<T, E = Error> = std::result::Result<T, E>;
