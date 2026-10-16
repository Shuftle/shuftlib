//! A generic, stack-like collection of cards: `draw` and `push` act on the same end,
//! plus shuffle and deal operations.
//!
//! [`Deck`](crate::cards::deck::Deck) is generic over any [`Card`] implementation; this
//! module also provides constructors for the standard Italian and French decks.

use rand::{Rng, RngExt, seq::SliceRandom};

use crate::cards::{
    Card, french,
    french::{FrenchCard, FrenchRank, FrenchWithJoker, Joker},
    italian,
    italian::{ItalianCard, ItalianRank},
};

#[derive(Default, Debug, Clone)]
/// Represents a deck of cards. Cards can be added or removed at will.
///
/// # Examples
///
/// Creating an Italian deck:
/// ```
/// use shuftlib::cards::{deck::Deck, italian::{ItalianRank, Suit}};
///
/// let deck = Deck::italian();
/// assert_eq!(deck.len(), 40);
/// assert!(deck.iter().any(|card| card.rank() == ItalianRank::Ace && card.suit() == Suit::Coins));
/// ```
///
/// Creating a French deck:
/// ```
/// use shuftlib::cards::{deck::Deck, french::{FrenchRank, Suit}};
///
/// let deck = Deck::french();
/// assert_eq!(deck.len(), 52);
/// assert!(deck.iter().any(|card| card.rank() == FrenchRank::Ace && card.suit() == Suit::Hearts));
/// ```
///
/// Creating a French deck with jokers:
/// ```
/// use shuftlib::cards::{deck::Deck, french::FrenchWithJoker};
///
/// let deck = Deck::french_with_jokers(2);
/// assert_eq!(deck.len(), 54);
/// assert_eq!(deck.iter().filter(|c| matches!(c, FrenchWithJoker::Joker(_))).count(), 2);
/// ```
pub struct Deck<T>
where
    T: Card,
{
    cards: Vec<T>,
}

const FRENCH_CARDS: usize = 52;
const ITALIAN_CARDS: usize = 40;

impl Deck<ItalianCard> {
    /// Creates a new deck in the Italian format.
    ///
    /// # Examples
    /// ```
    /// use shuftlib::cards::{deck::Deck, italian::ItalianCard};
    ///
    /// let deck = Deck::italian();
    /// assert_eq!(deck.len(), 40);
    /// ```
    pub fn italian() -> Deck<ItalianCard> {
        let mut cards = Vec::with_capacity(ITALIAN_CARDS);
        for suit in italian::Suit::ALL {
            for rank in ItalianRank::ALL {
                cards.push(ItalianCard::new(rank, suit));
            }
        }

        Deck { cards }
    }
}

impl Deck<FrenchCard> {
    /// Creates a new 52-card French deck.
    ///
    /// # Examples
    /// ```
    /// use shuftlib::cards::{deck::Deck, french::FrenchCard};
    ///
    /// let deck = Deck::french();
    /// assert_eq!(deck.len(), 52);
    /// ```
    pub fn french() -> Deck<FrenchCard> {
        let mut cards = Vec::with_capacity(FRENCH_CARDS);
        for suit in french::Suit::ALL {
            for rank in FrenchRank::ALL {
                cards.push(FrenchCard::new(rank, suit));
            }
        }

        Deck { cards }
    }
}

impl Deck<FrenchWithJoker> {
    /// Creates a new 52-card French deck, with the specified number of jokers added.
    ///
    /// # Examples
    /// ```
    /// use shuftlib::cards::{deck::Deck, french::{FrenchWithJoker, Joker}};
    ///
    /// let deck = Deck::french_with_jokers(2);
    /// assert_eq!(deck.len(), 54);
    /// assert_eq!(deck.iter().filter(|c| matches!(c, FrenchWithJoker::Joker(_))).count(), 2);
    /// ```
    pub fn french_with_jokers(jokers: u8) -> Deck<FrenchWithJoker> {
        let mut cards = Vec::with_capacity(FRENCH_CARDS + jokers as usize);
        for suit in french::Suit::ALL {
            for rank in FrenchRank::ALL {
                cards.push(FrenchWithJoker::Normal(FrenchCard::new(rank, suit)));
            }
        }

        for _ in 0..jokers {
            cards.push(FrenchWithJoker::Joker(Joker {}));
        }

        Deck { cards }
    }
}

impl<T: Card> Deck<T> {
    /// Performs a random permutation on the deck using the Fisher-Yates shuffle algorithm.
    ///
    /// Takes the random number generator to use, so callers that need reproducible
    /// shuffles (e.g. for deterministic replay) can supply a seeded one.
    ///
    /// # Examples
    /// ```
    /// use shuftlib::cards::{deck::Deck, italian::ItalianCard};
    ///
    /// let mut deck = Deck::italian();
    /// deck.shuffle(&mut rand::rng());
    /// assert_eq!(deck.len(), 40);
    /// ```
    pub fn shuffle(&mut self, rng: &mut impl Rng) {
        self.cards.shuffle(rng);
    }

    /// Adds a card in a random position inside the deck: neither on top nor
    /// at the bottom, every position between them equally likely. A deck of
    /// fewer than two cards has no inside, so the card goes on top.
    ///
    /// Takes the random number generator to use, so callers that need reproducible
    /// results (e.g. for deterministic replay) can supply a seeded one.
    ///
    /// # Examples
    /// ```
    /// use shuftlib::cards::{deck::Deck, italian::{ItalianCard, ItalianRank, Suit}};
    ///
    /// let mut deck = Deck::italian();
    /// let card = ItalianCard::new(ItalianRank::Ace, Suit::Coins);
    /// deck.shuffle_card(card, &mut rand::rng());
    /// assert_eq!(deck.len(), 41);
    /// ```
    pub fn shuffle_card(&mut self, card: T, rng: &mut impl Rng) {
        let len = self.cards.len();
        // Inserting at `len` would put the card on top, at 0 at the bottom.
        let pos = if len < 2 {
            len
        } else {
            rng.random_range(1..len)
        };
        self.cards.insert(pos, card);
    }

    /// Adds a card to the top of the deck.
    ///
    /// # Examples
    /// ```
    /// use shuftlib::cards::{deck::Deck, italian::{ItalianCard, ItalianRank, Suit}};
    ///
    /// let mut deck = Deck::new();
    /// let card = ItalianCard::new(ItalianRank::Ace, Suit::Coins);
    /// deck.push(card);
    /// assert_eq!(deck.len(), 1);
    /// ```
    pub fn push(&mut self, card: T) {
        self.cards.push(card);
    }

    /// Draws the top-most card in the deck. It returns None if there are no cards left.
    ///
    /// # Examples
    /// ```
    /// use shuftlib::cards::{deck::Deck, italian::ItalianCard};
    ///
    /// let mut deck = Deck::italian();
    /// let card = deck.draw();
    /// assert!(card.is_some());
    /// ```
    pub fn draw(&mut self) -> Option<T> {
        self.cards.pop()
    }

    /// Draws `n` cards from the top of the deck, if enough cards remain.
    ///
    /// Returns `Some` iterator over the drawn cards if the deck contains at least `n` cards,
    /// or `None` if there are not enough cards left (the deck is unchanged in that case).
    ///
    /// # Examples
    /// ```
    /// use shuftlib::cards::{deck::Deck, italian::ItalianCard};
    ///
    /// let mut deck = Deck::italian();
    /// let drawn = deck.draw_n(3).unwrap().collect::<Vec<_>>();
    /// assert_eq!(drawn.len(), 3);
    /// assert_eq!(deck.len(), 37);
    /// ```
    pub fn draw_n(&mut self, n: usize) -> Option<impl Iterator<Item = T>> {
        if self.cards.len() < n {
            None
        } else {
            let mut drawn = Vec::with_capacity(n);
            for _ in 0..n {
                drawn.push(self.cards.pop()?);
            }
            Some(drawn.into_iter())
        }
    }

    /// Creates a new empty deck.
    ///
    /// # Examples
    /// ```
    /// use shuftlib::cards::{deck::Deck, italian::ItalianCard};
    ///
    /// let deck: Deck<ItalianCard> = Deck::new();
    /// assert!(deck.is_empty());
    /// ```
    pub fn new() -> Deck<T> {
        Deck { cards: Vec::new() }
    }

    /// Creates a new empty deck with specified capacity.
    ///
    /// # Examples
    /// ```
    /// use shuftlib::cards::{deck::Deck, italian::ItalianCard};
    ///
    /// let deck: Deck<ItalianCard> = Deck::with_capacity(10);
    /// assert!(deck.is_empty());
    /// ```
    pub fn with_capacity(capacity: usize) -> Deck<T> {
        Deck {
            cards: Vec::with_capacity(capacity),
        }
    }

    /// Returns an iterator over the cards in the deck.
    ///
    /// # Examples
    /// ```
    /// use shuftlib::cards::{deck::Deck, italian::{ItalianCard, ItalianRank, Suit}};
    ///
    /// let cards = vec![
    ///     ItalianCard::new(ItalianRank::Ace, Suit::Coins),
    ///     ItalianCard::new(ItalianRank::Two, Suit::Coins),
    ///     ItalianCard::new(ItalianRank::Three, Suit::Coins),
    /// ];
    /// let deck: Deck<ItalianCard> = Deck::from(cards.clone());
    /// assert_eq!(deck.iter().count(), 3);
    /// assert_eq!(*deck.iter().next().unwrap(), cards[0]);
    /// ```
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.cards.iter()
    }

    /// Returns a slice of all cards in the deck.
    ///
    /// # Examples
    /// ```
    /// use shuftlib::cards::{deck::Deck, italian::ItalianCard};
    ///
    /// let deck = Deck::italian();
    /// let slice = deck.as_slice();
    /// assert_eq!(slice.len(), 40);
    /// ```
    pub fn as_slice(&self) -> &[T] {
        &self.cards
    }

    /// Returns the number of cards left in the deck.
    ///
    /// # Examples
    /// ```
    /// use shuftlib::cards::{deck::Deck, italian::{ItalianCard, ItalianRank, Suit}};
    ///
    /// let cards = vec![
    ///     ItalianCard::new(ItalianRank::Ace, Suit::Coins),
    ///     ItalianCard::new(ItalianRank::Two, Suit::Coins),
    ///     ItalianCard::new(ItalianRank::Three, Suit::Coins),
    /// ];
    /// let deck: Deck<ItalianCard> = Deck::from(cards);
    /// assert_eq!(deck.len(), 3);
    /// ```
    pub fn len(&self) -> usize {
        self.cards.len()
    }

    /// Returns whether or not the deck is empty.
    ///
    /// # Examples
    /// ```
    /// use shuftlib::cards::{deck::Deck, italian::ItalianCard};
    ///
    /// let deck: Deck<ItalianCard> = Deck::new();
    /// assert!(deck.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
}

impl<T: Card> From<Vec<T>> for Deck<T> {
    fn from(cards: Vec<T>) -> Self {
        Deck { cards }
    }
}

impl<T: Card> FromIterator<T> for Deck<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Deck {
            cards: Vec::from_iter(iter),
        }
    }
}

impl<T: Card> AsRef<[T]> for Deck<T> {
    fn as_ref(&self) -> &[T] {
        &self.cards
    }
}

impl<T: Card> IntoIterator for Deck<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.cards.into_iter()
    }
}

impl<'a, T: Card> IntoIterator for &'a Deck<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.cards.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::Deck;
    use crate::cards::deck::test_utils::deck_strategy;
    use crate::cards::italian::ItalianCard;
    use crate::cards::italian::test_utils::italian_rank_strategy;
    use crate::cards::italian::test_utils::suit_strategy;
    use std::collections::BTreeSet;

    use proptest::prelude::*;
    use rand::{SeedableRng, rngs::StdRng};

    #[test]
    fn shuffle_changes_order() {
        let sorted_deck = Deck::italian();

        let mut shuffled_deck = Deck::italian();
        shuffled_deck.shuffle(&mut rand::rng());

        assert_ne!(sorted_deck.as_slice(), shuffled_deck.as_slice());
    }

    /// Every position inside the deck, between the bottom and the top card,
    /// can receive the card.
    #[test]
    fn shuffle_card_can_reach_every_position_inside_the_deck() {
        let cards: Vec<ItalianCard> = Deck::italian().into_iter().collect();
        let (outsider, rest) = cards.split_last().unwrap();
        for len in 2..10 {
            let reached: BTreeSet<usize> = (0..500)
                .map(|seed| {
                    let mut deck = Deck::from(rest[..len].to_vec());
                    deck.shuffle_card(*outsider, &mut StdRng::seed_from_u64(seed));
                    deck.iter().position(|card| card == outsider).unwrap()
                })
                .collect();
            assert_eq!(reached, (1..len).collect(), "a deck of {len} cards");
        }
    }

    #[test]
    fn draw_n_basic() {
        let mut deck = Deck::italian();
        let drawn: Vec<_> = deck.draw_n(5).unwrap().collect();
        assert_eq!(drawn.len(), 5);
        assert_eq!(deck.len(), 35);

        // Drawing more than available returns None and does not change the deck
        let mut deck = Deck::italian();
        assert!(deck.draw_n(41).is_none());
        assert_eq!(deck.len(), 40);
    }

    proptest! {
        /// Pushing and drawing act on the same end of the deck, the top:
        /// the card pushed is the next one drawn, whatever is below it.
        #[test]
        fn push_and_draw_returns_same_card(
            card in (italian_rank_strategy(), suit_strategy()),
            mut deck in deck_strategy(0..40)
        ) {
            let card = ItalianCard::new(card.0, card.1);
            let below = deck.len();
            deck.push(card);
            prop_assert_eq!(deck.draw(), Some(card));
            prop_assert_eq!(deck.len(), below);
        }

        /// The card goes into the deck once, and, for decks of two cards or
        /// more, neither on top nor at the bottom.
        #[test]
        fn shuffle_card_inserts_the_card_inside_the_deck(
            card in (italian_rank_strategy(), suit_strategy()),
            mut deck in deck_strategy(0..40)
        ) {
            let card = ItalianCard::new(card.0, card.1);
            prop_assume!(!deck.as_slice().contains(&card));
            let old_len = deck.len();
            deck.shuffle_card(card, &mut rand::rng());
            prop_assert_eq!(deck.len(), old_len + 1);
            let positions: Vec<_> = deck
                .iter()
                .enumerate()
                .filter(|(_, other)| **other == card)
                .map(|(position, _)| position)
                .collect();
            prop_assert_eq!(positions.len(), 1);
            if old_len >= 2 {
                prop_assert!(positions[0] != 0 && positions[0] != old_len);
            }
        }
    }
}

/// Test utilities for the deck module.
#[cfg(test)]
pub mod test_utils {
    use std::ops::Range;

    use super::*;
    use crate::cards::italian::{ItalianCard, ItalianRank, Suit};
    use proptest::prelude::*;

    /// Generates a Deck<ItalianCard> with unique cards, of size in the given range.
    pub fn deck_strategy(size: Range<usize>) -> impl Strategy<Value = Deck<ItalianCard>> {
        // All possible unique cards
        let all_cards: Vec<ItalianCard> = ItalianRank::ALL
            .into_iter()
            .flat_map(|rank| Suit::ALL.map(|suit| ItalianCard::new(rank, suit)))
            .collect();

        proptest::sample::subsequence(all_cards, size).prop_map(Deck::from)
    }
}
