//! Italian playing cards.
//!
//! Provides [`ItalianCard`], [`ItalianRank`] and [`Suit`] for the 40-card
//! Italian deck: ace through seven, then jack, knight and king, in four
//! suits.

use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};

use crate::cards::Card;

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
    /// use shuftlib::cards::italian::{ItalianCard, ItalianRank, Suit};
    ///
    /// let card = ItalianCard::new(ItalianRank::Ace, Suit::Coins);
    /// assert_eq!(card.rank(), ItalianRank::Ace);
    /// assert_eq!(card.suit(), Suit::Coins);
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
    /// Formats the card as its rank and suit, e.g. "Ace of Coins".
    ///
    /// # Examples
    /// ```
    /// use shuftlib::cards::italian::{ItalianCard, ItalianRank, Suit};
    ///
    /// let card = ItalianCard::new(ItalianRank::Knight, Suit::Swords);
    /// assert_eq!(card.to_string(), "Knight of Swords");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} of {}", self.rank, self.suit)
    }
}

impl Card for ItalianCard {}

/// The four suits of the Italian deck.
///
/// Suits are ordered as listed: the deck order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Suit {
    /// Denari.
    Coins,
    /// Coppe.
    Cups,
    /// Spade.
    Swords,
    /// Bastoni.
    Clubs,
}

impl Suit {
    /// Every suit, in deck order.
    pub const ALL: [Self; 4] = [Self::Coins, Self::Cups, Self::Swords, Self::Clubs];
}

impl Display for Suit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Coins => "Coins",
            Self::Cups => "Cups",
            Self::Swords => "Swords",
            Self::Clubs => "Clubs",
        };
        f.write_str(name)
    }
}

/// A value for each suit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct PerSuit<T> {
    /// The value for Coins.
    pub coins: T,
    /// The value for Cups.
    pub cups: T,
    /// The value for Swords.
    pub swords: T,
    /// The value for Clubs.
    pub clubs: T,
}

impl<T> PerSuit<T> {
    /// Builds the values by calling `value` for each [`Suit`], in deck order,
    /// as in [`Suit::ALL`].
    pub fn from_fn(mut value: impl FnMut(Suit) -> T) -> Self {
        Self {
            coins: value(Suit::Coins),
            cups: value(Suit::Cups),
            swords: value(Suit::Swords),
            clubs: value(Suit::Clubs),
        }
    }
}

impl<T> Index<Suit> for PerSuit<T> {
    type Output = T;

    fn index(&self, suit: Suit) -> &T {
        match suit {
            Suit::Coins => &self.coins,
            Suit::Cups => &self.cups,
            Suit::Swords => &self.swords,
            Suit::Clubs => &self.clubs,
        }
    }
}

impl<T> IndexMut<Suit> for PerSuit<T> {
    fn index_mut(&mut self, suit: Suit) -> &mut T {
        match suit {
            Suit::Coins => &mut self.coins,
            Suit::Cups => &mut self.cups,
            Suit::Swords => &mut self.swords,
            Suit::Clubs => &mut self.clubs,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(u8)]
/// The rank of the card. In an Italian deck, ranks go from the ace to the 7, then they also have a jack, knight and king.
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

impl ItalianRank {
    /// Every rank, in the order of the numbers printed on the cards.
    pub const ALL: [Self; 10] = [
        Self::Ace,
        Self::Two,
        Self::Three,
        Self::Four,
        Self::Five,
        Self::Six,
        Self::Seven,
        Self::Jack,
        Self::Knight,
        Self::King,
    ];
}

impl Display for ItalianRank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Ace => "Ace",
            Self::Two => "Two",
            Self::Three => "Three",
            Self::Four => "Four",
            Self::Five => "Five",
            Self::Six => "Six",
            Self::Seven => "Seven",
            Self::Jack => "Jack",
            Self::Knight => "Knight",
            Self::King => "King",
        };
        f.write_str(name)
    }
}

#[cfg(test)]
/// Test utilities for the Italian card module.
pub mod test_utils {
    use super::{ItalianCard, ItalianRank, PerSuit, Suit};
    use proptest::array::uniform4;
    use proptest::prelude::*;
    use proptest::sample;

    /// Generates a random Italian rank strategy for property testing.
    pub fn italian_rank_strategy() -> impl Strategy<Value = ItalianRank> {
        sample::select(ItalianRank::ALL.to_vec())
    }

    /// Generates a random Italian suit strategy for property testing.
    pub fn suit_strategy() -> impl Strategy<Value = Suit> {
        sample::select(Suit::ALL.to_vec())
    }

    /// Generates a random Italian card strategy for property testing.
    pub fn italian_card_strategy() -> impl Strategy<Value = ItalianCard> {
        (italian_rank_strategy(), suit_strategy())
            .prop_map(|(rank, suit)| ItalianCard::new(rank, suit))
    }

    /// Generates a value for each suit, each drawn from `value`.
    pub fn per_suit_strategy<S: Strategy>(value: S) -> impl Strategy<Value = PerSuit<S::Value>> {
        uniform4(value).prop_map(|[coins, cups, swords, clubs]| PerSuit {
            coins,
            cups,
            swords,
            clubs,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{PerSuit, Suit};
    use crate::cards::deck::Deck;

    #[test]
    fn values_for_each_suit_are_built_in_deck_order() {
        let mut visited = Vec::new();
        let built = PerSuit::from_fn(|suit| {
            visited.push(suit);
            visited.len()
        });
        assert_eq!(visited, Suit::ALL);
        assert_eq!(built[Suit::Swords], 3);
    }

    #[test]
    fn the_deck_has_forty_distinct_cards_with_distinct_names() {
        let deck = Deck::italian();
        let cards: HashSet<_> = deck.iter().copied().collect();
        let names: HashSet<_> = deck.iter().map(ToString::to_string).collect();
        assert_eq!(deck.len(), 40);
        assert_eq!(cards.len(), 40);
        assert_eq!(names.len(), 40);
    }
}
