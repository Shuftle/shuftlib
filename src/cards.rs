//! Cards and decks, for any card game.
//!
//! Every deck names its own suits and ranks: [`italian`] for the 40-card
//! Italian deck, [`french`] for the 52-card French one.

use std::fmt::{Debug, Display};

pub mod french;
pub mod italian;

/// Generic deck container and operations.
pub mod deck;

/// A trait representing a card. The actual implementation depends on the game where this is used.
///
/// This trait ensures cards can be displayed, compared, and used in collections.
/// Implementations must provide equality based on card identity (e.g., rank and suit).
pub trait Card: Display + Default + Sized + Debug + Copy + Eq + PartialEq {}
