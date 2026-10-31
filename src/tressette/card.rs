//! Card implementation for the Tressette game.
//!
//! This module provides [`TressetteCard`], a wrapper around [`ItalianCard`](crate::cards::italian::ItalianCard)
//! that implements Tressette-specific ordering and scoring rules.

use std::{cmp::Ordering, fmt::Display, ops::Deref};

use crate::cards::italian::{ItalianCard, ItalianRank};
use crate::cards::{Card, italian::Suit};

/// A card in the Tressette game.
///
/// Wraps an [`ItalianCard`] and implements Tressette-specific ordering: cards
/// are ordered by suit, then by strength, which within a suit is
/// 4 < 5 < 6 < 7 < Jack < Knight < King < Ace < 2 < 3.
///
/// # Examples
///
/// ```
/// use shuftlib::{tressette::TressetteCard, cards::italian::{ItalianRank, Suit}};
///
/// let ace = TressetteCard::new(ItalianRank::Ace, Suit::Cups);
/// let three = TressetteCard::new(ItalianRank::Three, Suit::Cups);
///
/// assert!(three > ace);
/// assert_eq!(ace.suit(), Suit::Cups);
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
    /// Compares cards by suit, then by Tressette strength.
    ///
    /// Within a suit, the order from weakest to strongest is:
    /// 4 < 5 < 6 < 7 < Jack < Knight < King < Ace < 2 < 3. Only cards of the
    /// same suit compete for a trick, so the order between suits is just
    /// the deck order, to keep the ordering consistent with equality.
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::{tressette::TressetteCard, cards::italian::{ItalianRank, Suit}};
    ///
    /// // Test all adjacent pairs (transitivity guarantees full ordering)
    /// let four = TressetteCard::new(ItalianRank::Four, Suit::Cups);
    /// let five = TressetteCard::new(ItalianRank::Five, Suit::Cups);
    /// let six = TressetteCard::new(ItalianRank::Six, Suit::Cups);
    /// let seven = TressetteCard::new(ItalianRank::Seven, Suit::Cups);
    /// let jack = TressetteCard::new(ItalianRank::Jack, Suit::Cups);
    /// let knight = TressetteCard::new(ItalianRank::Knight, Suit::Cups);
    /// let king = TressetteCard::new(ItalianRank::King, Suit::Cups);
    /// let ace = TressetteCard::new(ItalianRank::Ace, Suit::Cups);
    /// let two = TressetteCard::new(ItalianRank::Two, Suit::Cups);
    /// let three = TressetteCard::new(ItalianRank::Three, Suit::Cups);
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

        (self.suit(), rank_order(self.rank())).cmp(&(other.suit(), rank_order(other.rank())))
    }
}

impl Display for TressetteCard {
    /// Formats the card as its rank and suit, e.g. "Ace of Cups".
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::{tressette::TressetteCard, cards::italian::{ItalianRank, Suit}};
    ///
    /// let card = TressetteCard::new(ItalianRank::Ace, Suit::Cups);
    /// assert_eq!(format!("{}", card), "Ace of Cups");
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
    /// use shuftlib::{tressette::TressetteCard, cards::italian::{ItalianCard, ItalianRank, Suit}};
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
    /// Returns the value of the card in Tressette, in thirds of a point:
    ///
    /// - Ace = 3 thirds, a whole point;
    /// - Two, Three, and figures (Jack, Knight, King) = 1 third;
    /// - All other cards = 0.
    ///
    /// Counting in thirds keeps the sums exact: a team scores the whole
    /// points among the thirds it took.
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::{tressette::TressetteCard, cards::italian::{ItalianRank, Suit}};
    ///
    /// let ace = TressetteCard::new(ItalianRank::Ace, Suit::Cups);
    /// let two = TressetteCard::new(ItalianRank::Two, Suit::Swords);
    /// let four = TressetteCard::new(ItalianRank::Four, Suit::Clubs);
    ///
    /// assert_eq!(ace.thirds(), 3);
    /// assert_eq!(two.thirds(), 1);
    /// assert_eq!(four.thirds(), 0);
    /// ```
    pub fn thirds(&self) -> u8 {
        match self.rank() {
            ItalianRank::Ace => 3,
            ItalianRank::Two
            | ItalianRank::Three
            | ItalianRank::Jack
            | ItalianRank::Knight
            | ItalianRank::King => 1,
            ItalianRank::Four | ItalianRank::Five | ItalianRank::Six | ItalianRank::Seven => 0,
        }
    }

    /// Creates a new `TressetteCard` from a rank and suit.
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::{cards::italian::{ItalianCard, ItalianRank, Suit}, tressette::TressetteCard};
    ///
    /// let card = TressetteCard::new(ItalianRank::Ace, Suit::Swords);
    /// assert_eq!(*card, ItalianCard::new(ItalianRank::Ace, Suit::Swords));
    /// ```
    pub fn new(rank: ItalianRank, suit: Suit) -> Self {
        let card = ItalianCard::new(rank, suit);

        TressetteCard { card }
    }
}

#[cfg(test)]
/// Test utilities for the Tressette card module.
pub mod test_utils {
    use super::TressetteCard;
    use crate::cards::italian::test_utils::italian_card_strategy;
    use crate::tressette::deck;
    use proptest::{prelude::*, sample};

    /// Generates a random Tressette card strategy for property testing.
    pub fn tressette_card_strategy() -> impl Strategy<Value = TressetteCard> {
        italian_card_strategy().prop_map(TressetteCard::from)
    }

    /// Generates any set of distinct Tressette cards, from none to the whole
    /// deck.
    pub fn cards_strategy() -> impl Strategy<Value = Vec<TressetteCard>> {
        let all: Vec<_> = deck().into_iter().collect();
        let size = all.len();
        sample::subsequence(all, 0..=size)
    }
}
