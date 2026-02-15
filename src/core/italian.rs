use std::fmt::Display;

use strum::{EnumIter, FromRepr};

use crate::core::{Card, Suit};

/// Representation of a card that goes into an Italian deck.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ItalianCard {
    rank: ItalianRank,
    suit: Suit,
}

impl ItalianCard {
    /// Generates a card with the given rank and suit
    ///
    /// # Examples
    /// ```
    /// use shuftlib::core::italian::{ItalianCard, ItalianRank};
    /// use shuftlib::core::Suit;
    ///
    /// let card = ItalianCard::new(ItalianRank::Ace, Suit::Hearts);
    /// assert_eq!(card.rank(), ItalianRank::Ace);
    /// assert_eq!(card.suit(), Suit::Hearts);
    /// ```
    pub fn new(rank: ItalianRank, suit: Suit) -> Self {
        Self { rank, suit }
    }

    /// The rank of the card.
    pub fn rank(&self) -> ItalianRank {
        self.rank
    }

    /// The suit of the card.
    pub fn suit(&self) -> Suit {
        self.suit
    }
}

impl Default for ItalianCard {
    fn default() -> Self {
        ItalianCard {
            rank: ItalianRank::Ace,
            suit: Suit::Clubs,
        }
    }
}

impl Display for ItalianCard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.rank as u8, self.suit)
    }
}

impl Card for ItalianCard {}

#[derive(Debug, Clone, Copy, Eq, PartialEq, EnumIter, FromRepr, Hash)]
#[repr(u8)]
/// The rank of the card. In an Italian deck, ranks go from the ace to the 7, then they also have a jack, knight and king,
/// In most games they each have a different value that depends on the game itself.
pub enum ItalianRank {
    /// 1
    Ace = 1,
    /// 2
    Two,
    /// 3
    Three,
    /// 4
    Four,
    /// 5
    Five,
    /// 6
    Six,
    /// 7
    Seven,
    /// 8
    Jack,
    /// 9
    Knight,
    /// 10
    King,
}

#[cfg(test)]
/// Test utilities for the Italian card module.
pub mod test_utils {
    use super::ItalianRank;
    use proptest::prelude::*;

    /// Generates a random Italian rank strategy for property testing.
    pub fn italian_rank_strategy() -> impl Strategy<Value = ItalianRank> {
        prop_oneof![
            Just(ItalianRank::Ace),
            Just(ItalianRank::Two),
            Just(ItalianRank::Three),
            Just(ItalianRank::Four),
            Just(ItalianRank::Five),
            Just(ItalianRank::Six),
            Just(ItalianRank::Seven),
            Just(ItalianRank::Jack),
            Just(ItalianRank::Knight),
            Just(ItalianRank::King),
        ]
    }
}
