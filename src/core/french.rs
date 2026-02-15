use std::fmt::Display;

use strum::{EnumIter, FromRepr};

use crate::core::{Card, Suit};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Representation of a card that goes into an French deck.
pub struct FrenchCard {
    rank: FrenchRank,
    suit: Suit,
}

impl FrenchCard {
    /// Generates a card with the given rank and suit
    ///
    /// # Examples
    /// ```
    /// use shuftlib::core::french::{FrenchCard, FrenchRank};
    /// use shuftlib::core::Suit;
    ///
    /// let card = FrenchCard::new(FrenchRank::Ace, Suit::Hearts);
    /// assert_eq!(card.rank(), FrenchRank::Ace);
    /// assert_eq!(card.suit(), Suit::Hearts);
    /// ```
    pub fn new(rank: FrenchRank, suit: Suit) -> Self {
        Self { rank, suit }
    }

    /// The rank of the card.
    pub fn rank(&self) -> FrenchRank {
        self.rank
    }

    /// The suit of the card.
    pub fn suit(&self) -> Suit {
        self.suit
    }
}

impl Default for FrenchCard {
    fn default() -> Self {
        FrenchCard {
            rank: FrenchRank::Ace,
            suit: Suit::Hearts,
        }
    }
}

impl Display for FrenchCard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.rank as u8, self.suit)
    }
}

impl Card for FrenchCard {}

/// A Joker card, present in some card games. Its function depends on the game.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Joker;

impl Card for Joker {}

impl Display for Joker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "JK")
    }
}

/// A variant of the French card, which can either be an actual French card or a joker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrenchWithJoker {
    /// A standard French card.
    Normal(FrenchCard),
    /// A Joker card.
    Joker(Joker),
}
impl Card for FrenchWithJoker {}

impl Default for FrenchWithJoker {
    fn default() -> Self {
        Self::Normal(FrenchCard {
            rank: FrenchRank::Ace,
            suit: Suit::Hearts,
        })
    }
}

impl Display for FrenchWithJoker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FrenchWithJoker::Normal(c) => write!(f, "{c}"),
            FrenchWithJoker::Joker(c) => write!(f, "{c}"),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, EnumIter, FromRepr, Hash)]
#[repr(u8)]
/// The rank of the card. In a French deck, ranks go from the ace to 10, then there is a jack, queen and king,
/// In most games they each have a different value that depends on the game itself.
pub enum FrenchRank {
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
    Eight,
    /// 9
    Nine,
    /// 10
    Ten,
    /// 11
    Jack,
    /// 12
    Queen,
    /// 13
    King,
}
