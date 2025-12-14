//! Core primitives for card games.
//!
//! This module contains the fundamental building blocks for card games:
//! - Generic card types, suits, ranks, and decks (applicable to any card game)
//! - Trick-taking game mechanics (players, tricks, hands)

use std::fmt::{Debug, Display};
use strum::EnumIter;

/// Italian card deck types and ranks.
pub mod italian;

/// French card deck types, ranks, and Joker.
pub mod french;

/// Generic deck container and operations.
pub mod deck;

/// Trick-taking game mechanics including players, tricks, and hands.
/// Contains specific types and traits for trick-taking games.
pub use crate::trick_taking;

/// A trait representing a card. The actual implementation depends on the game where this is used.
///
/// This trait ensures cards can be displayed, compared, and used in collections.
/// Implementations must provide equality based on card identity (e.g., rank and suit).
pub trait Card: Display + Default + Sized + Debug + Copy + Eq + PartialEq {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, Hash)]
/// The 4 suits of a standard deck. They have an equivalent in pretty much all regional decks.
/// In some games they have a hierarchical order.
///
/// # Examples
/// ```
/// use shuftlib::core::Suit;
///
/// let heart = Suit::Hearts;
/// assert_eq!(format!("{}", heart), "H");
/// ```
pub enum Suit {
    /// Hearts (French, German), Cups (Latin).
    Hearts,
    /// Diamonds or Tiles (French), Bells (German), Coins (Latin).
    Diamonds,
    /// Clubs or Clover (French), Acorns (German), Clubs or Batons (Latin).
    Clubs,
    /// Spades or Pikes (French), Leaves (German), Swords (Latin).
    Spades,
}

impl Display for Suit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Suit::Hearts => "H",
            Suit::Diamonds => "D",
            Suit::Clubs => "C",
            Suit::Spades => "S",
        };
        write!(f, "{s}")
    }
}

/// Test utilities for the core module.
#[cfg(test)]
pub mod test_utils {
    use super::Suit;
    use proptest::prelude::*;

    /// Generates a random suit strategy for property testing.
    pub fn suit_strategy() -> impl Strategy<Value = Suit> {
        prop_oneof![
            Just(Suit::Hearts),
            Just(Suit::Diamonds),
            Just(Suit::Clubs),
            Just(Suit::Spades),
        ]
    }
}
