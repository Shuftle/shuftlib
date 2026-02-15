//! Card implementation for the Tressette game.
//!
//! This module provides [`TressetteCard`], a wrapper around [`ItalianCard`] that implements
//! Tressette-specific ordering and scoring rules.

use std::{cmp::Ordering, fmt::Display, ops::Deref};

use crate::core::italian::{ItalianCard, ItalianRank};
use crate::core::{Card, Suit};
use num_rational::Rational32;

/// A card in the Tressette game.
///
/// Wraps an [`ItalianCard`] and implements Tressette-specific ordering where:
/// 4 < 5 < 6 < 7 < Jack < Knight < King < Ace < 2 < 3
///
/// # Examples
///
/// ```
/// use shuftlib::tressette::TressetteCard;
/// use shuftlib::core::Suit;
/// use shuftlib::core::italian::ItalianRank;
///
/// let ace = TressetteCard::new(ItalianRank::Ace, Suit::Hearts);
/// let three = TressetteCard::new(ItalianRank::Three, Suit::Hearts);
///
/// assert!(three > ace);
/// assert_eq!(ace.suit(), Suit::Hearts);
/// ```
#[derive(PartialEq, Eq, Debug, Clone, Copy, Default, Hash)]
pub struct TressetteCard {
    card: ItalianCard,
}

impl PartialOrd for TressetteCard {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TressetteCard {
    /// Compare cards by Tressette rank ordering.
    ///
    /// In Tressette, the rank order from lowest to highest is:
    /// 4 < 5 < 6 < 7 < Jack < Knight < King < Ace < 2 < 3
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::tressette::TressetteCard;
    /// use shuftlib::core::Suit;
    /// use shuftlib::core::italian::ItalianRank;
    ///
    /// // Test all adjacent pairs (transitivity guarantees full ordering)
    /// let four = TressetteCard::new(ItalianRank::Four, Suit::Hearts);
    /// let five = TressetteCard::new(ItalianRank::Five, Suit::Hearts);
    /// let six = TressetteCard::new(ItalianRank::Six, Suit::Hearts);
    /// let seven = TressetteCard::new(ItalianRank::Seven, Suit::Hearts);
    /// let jack = TressetteCard::new(ItalianRank::Jack, Suit::Hearts);
    /// let knight = TressetteCard::new(ItalianRank::Knight, Suit::Hearts);
    /// let king = TressetteCard::new(ItalianRank::King, Suit::Hearts);
    /// let ace = TressetteCard::new(ItalianRank::Ace, Suit::Hearts);
    /// let two = TressetteCard::new(ItalianRank::Two, Suit::Hearts);
    /// let three = TressetteCard::new(ItalianRank::Three, Suit::Hearts);
    ///
    /// assert!(four < five);
    /// assert!(five < six);
    /// assert!(six < seven);
    /// assert!(seven < jack);
    /// assert!(jack < knight);
    /// assert!(knight < king);
    /// assert!(king < ace);
    /// assert!(ace < two);
    /// assert!(two < three);
    /// ```
    fn cmp(&self, other: &Self) -> Ordering {
        fn rank_order(rank: ItalianRank) -> u8 {
            match rank {
                ItalianRank::Four => 0,
                ItalianRank::Five => 1,
                ItalianRank::Six => 2,
                ItalianRank::Seven => 3,
                ItalianRank::Jack => 4,
                ItalianRank::Knight => 5,
                ItalianRank::King => 6,
                ItalianRank::Ace => 7,
                ItalianRank::Two => 8,
                ItalianRank::Three => 9,
            }
        }

        rank_order(self.rank()).cmp(&rank_order(other.rank()))
    }
}

impl Display for TressetteCard {
    /// Formats the card for display using rank number and suit symbol.
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::tressette::TressetteCard;
    /// use shuftlib::core::Suit;
    /// use shuftlib::core::italian::ItalianRank;
    ///
    /// let card = TressetteCard::new(ItalianRank::Ace, Suit::Hearts);
    /// assert_eq!(format!("{}", card), "1H");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.card)
    }
}

impl Card for TressetteCard {}

impl From<ItalianCard> for TressetteCard {
    /// Creates a `TressetteCard` from an `ItalianCard`.
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::tressette::TressetteCard;
    /// use shuftlib::core::Suit;
    /// use shuftlib::core::italian::{ItalianCard, ItalianRank};
    ///
    /// let italian_card = ItalianCard::new(ItalianRank::King, Suit::Clubs);
    /// let tressette_card: TressetteCard = italian_card.into();
    /// assert_eq!(*tressette_card, italian_card);
    /// ```
    fn from(value: ItalianCard) -> Self {
        TressetteCard { card: value }
    }
}

impl Deref for TressetteCard {
    type Target = ItalianCard;

    fn deref(&self) -> &Self::Target {
        &self.card
    }
}

impl TressetteCard {
    /// Returns the point value of the card in Tressette.
    ///
    /// - Ace = 1 point (3/3)
    /// - Two, Three, and figures (Jack, Knight, King) = 1/3 point
    /// - All other cards = 0 points
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::tressette::TressetteCard;
    /// use shuftlib::core::Suit;
    /// use shuftlib::core::italian::ItalianRank;
    /// use num_rational::Rational32;
    ///
    /// let ace = TressetteCard::new(ItalianRank::Ace, Suit::Hearts);
    /// let two = TressetteCard::new(ItalianRank::Two, Suit::Spades);
    /// let four = TressetteCard::new(ItalianRank::Four, Suit::Clubs);
    ///
    /// assert_eq!(ace.value(), Rational32::new(3, 3));
    /// assert_eq!(two.value(), Rational32::new(1, 3));
    /// assert_eq!(four.value(), Rational32::new(0, 3));
    /// ```
    pub fn value(&self) -> Rational32 {
        match self.rank() {
            ItalianRank::Ace => Rational32::new(3, 3),
            ItalianRank::Two
            | ItalianRank::Three
            | ItalianRank::King
            | ItalianRank::Knight
            | ItalianRank::Jack => Rational32::new(1, 3),
            ItalianRank::Four | ItalianRank::Five | ItalianRank::Six | ItalianRank::Seven => {
                Rational32::new(0, 3)
            }
        }
    }

    /// Creates a new `TressetteCard` from a rank and suit.
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::core::Suit;
    /// use shuftlib::core::italian::{ItalianCard, ItalianRank};
    /// use shuftlib::tressette::TressetteCard;
    ///
    /// let card = TressetteCard::new(ItalianRank::Ace, Suit::Spades);
    /// assert_eq!(*card, ItalianCard::new(ItalianRank::Ace, Suit::Spades));
    /// ```
    pub fn new(rank: ItalianRank, suit: Suit) -> Self {
        let card = ItalianCard::new(rank, suit);

        TressetteCard { card }
    }
}
