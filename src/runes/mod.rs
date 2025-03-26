use super::*;

pub use {edict::Edict, rune::Rune, rune_id::RuneId, runestone::Runestone};

pub(crate) use {etching::Etching, pile::Pile, spaced_rune::SpacedRune};

pub const MAX_DIVISIBILITY: u8 = 38;
pub(crate) const CLAIM_BIT: u128 = 1 << 48;
pub(crate) const MAX_LIMIT: u128 = 1 << 64;
const RESERVED: u128 = 6402364363415443603228541259936211926;

mod edict;
mod etching;
mod pile;
mod rune;
mod rune_id;
mod runestone;
mod spaced_rune;
pub mod varint;

type Result<T, E = Error> = std::result::Result<T, E>;
