//! French playing cards.
//!
//! Provides [`FrenchCard`], [`FrenchRank`] and [`Suit`] for the standard
//! 52-card French deck, plus an optional [`Joker`] via [`FrenchWithJoker`].

use std::fmt::Display;

use crate::cards::Card;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Representation of a card that goes into a French deck.
pub struct FrenchCard {
    rank: FrenchRank,
    suit: Suit,
}

impl FrenchCard {
    /// Generates a card with the given rank and suit
    ///
    /// # Examples
    /// ```
    /// use shuftlib::cards::french::{FrenchCard, FrenchRank, Suit};
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
    /// Formats the card as its rank and suit, e.g. "Queen of Hearts".
    ///
    /// # Examples
    /// ```
    /// use shuftlib::cards::french::{FrenchCard, FrenchRank, Suit};
    ///
    /// let card = FrenchCard::new(FrenchRank::Queen, Suit::Hearts);
    /// assert_eq!(card.to_string(), "Queen of Hearts");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} of {}", self.rank, self.suit)
    }
}

impl Card for FrenchCard {}

/// A Joker card, present in some card games. Its function depends on the game.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Joker;

impl Card for Joker {}

impl Display for Joker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Joker")
    }
}

/// A variant of the French card, which can either be an actual French card or a joker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(u8)]
/// The rank of the card. In a French deck, ranks go from the ace to 10, then there is a jack, queen and king.
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

impl FrenchRank {
    /// Every rank, from the ace to the king.
    pub const ALL: [Self; 13] = [
        Self::Ace,
        Self::Two,
        Self::Three,
        Self::Four,
        Self::Five,
        Self::Six,
        Self::Seven,
        Self::Eight,
        Self::Nine,
        Self::Ten,
        Self::Jack,
        Self::Queen,
        Self::King,
    ];
}

impl Display for FrenchRank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Ace => "Ace",
            Self::Two => "Two",
            Self::Three => "Three",
            Self::Four => "Four",
            Self::Five => "Five",
            Self::Six => "Six",
            Self::Seven => "Seven",
            Self::Eight => "Eight",
            Self::Nine => "Nine",
            Self::Ten => "Ten",
            Self::Jack => "Jack",
            Self::Queen => "Queen",
            Self::King => "King",
        };
        f.write_str(name)
    }
}

/// The four suits of the French deck.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Suit {
    /// Hearts.
    Hearts,
    /// Diamonds, also called tiles.
    Diamonds,
    /// Clubs, also called clovers.
    Clubs,
    /// Spades, also called pikes.
    Spades,
}

impl Suit {
    /// Every suit, in deck order.
    pub const ALL: [Self; 4] = [Self::Hearts, Self::Diamonds, Self::Clubs, Self::Spades];
}

impl Display for Suit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Hearts => "Hearts",
            Self::Diamonds => "Diamonds",
            Self::Clubs => "Clubs",
            Self::Spades => "Spades",
        };
        f.write_str(name)
    }
}
