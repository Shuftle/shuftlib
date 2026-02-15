use rand::{RngExt, seq::SliceRandom};
use strum::IntoEnumIterator;

use crate::core::{
    Card, Suit,
    french::{FrenchCard, FrenchRank, FrenchWithJoker, Joker},
    italian::{ItalianCard, ItalianRank},
};

#[derive(Default, Debug, Clone)]
/// Represents a deck of cards. Cards can be added or removed at will.
///
/// # Examples
///
/// Creating an Italian deck:
/// ```
/// use shuftlib::core::deck::Deck;
/// use shuftlib::core::italian::{ItalianCard, ItalianRank};
/// use shuftlib::core::Suit;
///
/// let deck = Deck::<ItalianCard>::italian();
/// assert_eq!(deck.len(), 40);
/// assert!(deck.iter().any(|card| card.rank() == ItalianRank::Ace && card.suit() == Suit::Hearts));
/// ```
///
/// Creating a French deck:
/// ```
/// use shuftlib::core::deck::Deck;
/// use shuftlib::core::french::{FrenchCard, FrenchRank};
/// use shuftlib::core::Suit;
///
/// let deck = Deck::<FrenchCard>::french();
/// assert_eq!(deck.len(), 52);
/// assert!(deck.iter().any(|card| card.rank() == FrenchRank::Ace && card.suit() == Suit::Hearts));
/// ```
///
/// Creating a French deck with jokers:
/// ```
/// use shuftlib::core::deck::Deck;
/// use shuftlib::core::french::{FrenchWithJoker, Joker};
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
    /// Creates a new deck in the Italian format.
    ///
    /// # Examples
    /// ```
    /// use shuftlib::core::deck::Deck;
    /// use shuftlib::core::italian::ItalianCard;
    ///
    /// let deck = Deck::italian();
    /// assert_eq!(deck.len(), 40);
    /// ```
    pub fn italian() -> Deck<ItalianCard> {
        let mut cards = Vec::with_capacity(ITALIAN_CARDS);
        for suit in Suit::iter() {
            for rank in ItalianRank::iter() {
                cards.push(ItalianCard::new(rank, suit));
            }
        }

        Deck { cards }
    }
}

impl Deck<FrenchCard> {
    /// Creates a new 52 cards French deck.
    /// Creates a new 52 cards French deck.
    ///
    /// # Examples
    /// ```
    /// use shuftlib::core::deck::Deck;
    /// use shuftlib::core::french::FrenchCard;
    ///
    /// let deck = Deck::french();
    /// assert_eq!(deck.len(), 52);
    /// ```
    pub fn french() -> Deck<FrenchCard> {
        let mut cards = Vec::with_capacity(FRENCH_CARDS);
        for suit in Suit::iter() {
            for rank in FrenchRank::iter() {
                cards.push(FrenchCard::new(rank, suit));
            }
        }

        Deck { cards }
    }

    /// Creates a new 52 cards French deck, with the addition of the specified amount of jokers.
    /// Creates a new 52 cards French deck, with the addition of the specified amount of jokers.
    ///
    /// # Examples
    /// ```
    /// use shuftlib::core::deck::Deck;
    /// use shuftlib::core::french::{FrenchWithJoker, Joker};
    ///
    /// let deck = Deck::french_with_jokers(2);
    /// assert_eq!(deck.len(), 54);
    /// assert_eq!(deck.iter().filter(|c| matches!(c, FrenchWithJoker::Joker(_))).count(), 2);
    /// ```
    pub fn french_with_jokers(jokers: u8) -> Deck<FrenchWithJoker> {
        let mut cards = Vec::with_capacity(FRENCH_CARDS + jokers as usize);
        for suit in Suit::iter() {
            for rank in FrenchRank::iter() {
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
    /// # Examples
    /// ```
    /// use shuftlib::core::deck::Deck;
    /// use shuftlib::core::italian::ItalianCard;
    ///
    /// let mut deck = Deck::italian();
    /// deck.shuffle();
    /// assert_eq!(deck.len(), 40);
    /// ```
    pub fn shuffle(&mut self) {
        let mut rng = rand::rng();
        self.cards.shuffle(&mut rng);
    }

    /// Adds a card in a random position inside the deck.
    ///
    /// # Examples
    /// ```
    /// use shuftlib::core::deck::Deck;
    /// use shuftlib::core::italian::{ItalianCard, ItalianRank};
    /// use shuftlib::core::Suit;
    ///
    /// let mut deck = Deck::italian();
    /// let card = ItalianCard::new(ItalianRank::Ace, Suit::Hearts);
    /// deck.shuffle_card(card);
    /// assert!(deck.len() >= 41);
    /// ```
    pub fn shuffle_card(&mut self, card: T) {
        let len = self.cards.len();
        let mut rng = rand::rng();
        let pos = match len {
            0 => 0,
            1 => 1,
            2 => 1,
            _ => rng.random_range(1..len - 1),
        };
        self.cards.insert(pos, card);
    }

    /// Adds a card to the top of the deck.
    ///
    /// # Examples
    /// ```
    /// use shuftlib::core::deck::Deck;
    /// use shuftlib::core::italian::{ItalianCard, ItalianRank};
    /// use shuftlib::core::Suit;
    ///
    /// let mut deck = Deck::new();
    /// let card = ItalianCard::new(ItalianRank::Ace, Suit::Hearts);
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
    /// use shuftlib::core::deck::Deck;
    /// use shuftlib::core::italian::ItalianCard;
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
    /// use shuftlib::core::deck::Deck;
    /// use shuftlib::core::italian::ItalianCard;
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
    /// use shuftlib::core::deck::Deck;
    /// use shuftlib::core::italian::ItalianCard;
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
    /// use shuftlib::core::deck::Deck;
    /// use shuftlib::core::italian::ItalianCard;
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
    /// use shuftlib::core::deck::Deck;
    /// use shuftlib::core::italian::{ItalianCard, ItalianRank};
    /// use shuftlib::core::Suit;
    ///
    /// let cards = vec![
    ///     ItalianCard::new(ItalianRank::Ace, Suit::Hearts),
    ///     ItalianCard::new(ItalianRank::Two, Suit::Hearts),
    ///     ItalianCard::new(ItalianRank::Three, Suit::Hearts),
    /// ];
    /// let deck: Deck<ItalianCard> = Deck::from(cards.clone());
    /// assert_eq!(deck.iter().count(), 3);
    /// assert_eq!(*deck.iter().next().unwrap(), cards[0]);
    /// ```
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.cards.iter()
    }

    /// Returns a mutable iterator over the cards in the deck.
    ///
    /// # Examples
    /// ```
    /// use shuftlib::core::deck::Deck;
    /// use shuftlib::core::italian::{ItalianCard, ItalianRank};
    /// use shuftlib::core::Suit;
    ///
    /// let mut deck = Deck::from(vec![
    ///     ItalianCard::new(ItalianRank::Ace, Suit::Hearts),
    ///     ItalianCard::new(ItalianRank::Two, Suit::Hearts),
    /// ]);
    /// for card in deck.iter_mut() {
    ///     // Modify cards if needed
    /// }
    /// assert_eq!(deck.len(), 2);
    /// ```
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.cards.iter_mut()
    }

    /// Returns a slice of all cards in the deck.
    ///
    /// # Examples
    /// ```
    /// use shuftlib::core::deck::Deck;
    /// use shuftlib::core::italian::ItalianCard;
    ///
    /// let deck = Deck::italian();
    /// let slice = deck.as_slice();
    /// assert_eq!(slice.len(), 40);
    /// ```
    pub fn as_slice(&self) -> &[T] {
        &self.cards
    }

    /// Returns a mutable slice of all cards in the deck.
    ///
    /// # Examples
    /// ```
    /// use shuftlib::core::deck::Deck;
    /// use shuftlib::core::italian::ItalianCard;
    ///
    /// let mut deck = Deck::italian();
    /// let slice = deck.as_mut_slice();
    /// // Modify the slice if needed
    /// assert_eq!(slice.len(), 40);
    /// ```
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.cards
    }

    /// Returns the number of cards left in the deck.
    ///
    /// # Examples
    /// ```
    /// use shuftlib::core::deck::Deck;
    /// use shuftlib::core::italian::{ItalianCard, ItalianRank};
    /// use shuftlib::core::Suit;
    ///
    /// let cards = vec![
    ///     ItalianCard::new(ItalianRank::Ace, Suit::Hearts),
    ///     ItalianCard::new(ItalianRank::Two, Suit::Hearts),
    ///     ItalianCard::new(ItalianRank::Three, Suit::Hearts),
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
    /// use shuftlib::core::deck::Deck;
    /// use shuftlib::core::italian::ItalianCard;
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

impl<'a, T: Card> IntoIterator for &'a mut Deck<T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.cards.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::Deck;
    use crate::core::deck::test_utils::deck_strategy;
    use crate::core::italian::ItalianCard;
    use crate::core::italian::test_utils::italian_rank_strategy;
    use crate::core::test_utils::suit_strategy;
    use proptest::prelude::*;

    #[test]
    fn shuffle_changes_order() {
        let sorted_deck = Deck::italian();

        let mut shuffled_deck = Deck::italian();
        shuffled_deck.shuffle();

        assert_ne!(sorted_deck.as_slice(), shuffled_deck.as_slice());
    }

    #[test]
    fn shuffle_empty_deck() {
        let mut deck: Deck<ItalianCard> = Deck::new();
        deck.shuffle();
        assert!(deck.is_empty());
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
        #[test]
        fn shuffle_preserves_cards(
            mut deck in deck_strategy(0..=39)
        ) {
            let original: Vec<ItalianCard> = deck.as_slice().to_vec();
            deck.shuffle();
            let shuffled: Vec<ItalianCard> = deck.as_slice().to_vec();

            // Same length and same set of cards (order may differ)
            prop_assert_eq!(original.len(), shuffled.len());
            let mut original_sorted = original.clone();
            let mut shuffled_sorted = shuffled.clone();
            original_sorted.sort_by_key(|c: &ItalianCard| (c.rank() as u8, c.suit() as u8));
            shuffled_sorted.sort_by_key(|c: &ItalianCard| (c.rank() as u8, c.suit() as u8));
            prop_assert_eq!(original_sorted, shuffled_sorted);
        }

        #[test]
        fn draw_all_cards_yields_unique(
            mut deck in deck_strategy(0..=39)
        ) {
            let mut seen = std::collections::HashSet::new();
            while let Some(card) = deck.draw() {
                prop_assert!(seen.insert(card));
            }
        }

        #[test]
        fn push_and_draw_returns_same_card(
            card in (italian_rank_strategy(), suit_strategy())
        ) {
            let mut deck = Deck::new();
            let card = ItalianCard::new(card.0, card.1);
            deck.push(card);
            prop_assert_eq!(deck.draw(), Some(card));
        }

        #[test]
        fn shuffle_card_inserts_card(
            card in (italian_rank_strategy(), suit_strategy()),
            mut deck in deck_strategy(0..=39)
        ) {
            let card = ItalianCard::new(card.0, card.1);
            let old_len: usize = deck.len();
            deck.shuffle_card(card);
            prop_assert!(deck.as_slice().contains(&card));
            prop_assert_eq!(deck.len(), old_len + 1);
        }

        #[test]
        fn shuffle_card_never_inserts_at_top_or_bottom_for_large_deck(
            mut deck in deck_strategy(2..=40),
            card in (italian_rank_strategy(), suit_strategy())
        ) {
            let card = ItalianCard::new(card.0, card.1);
            prop_assume!(!deck.as_slice().contains(&card));
            deck.shuffle_card(card);
            let pos = deck.as_slice().iter().position(|c| *c == card).unwrap();
            let new_len = deck.len();
            prop_assert!(pos != 0 && pos != new_len - 1);
        }
    }
}

/// Test utilities for the deck module.
#[cfg(test)]
pub mod test_utils {
    use super::*;
    use crate::core::Suit;
    use crate::core::italian::{ItalianCard, ItalianRank};
    use proptest::prelude::*;

    /// Generates a Deck<ItalianCard> with unique cards, of size in the given range.
    pub fn deck_strategy(
        size: std::ops::RangeInclusive<usize>,
    ) -> impl Strategy<Value = Deck<ItalianCard>> {
        // All possible unique cards
        let all_cards: Vec<ItalianCard> = [
            ItalianRank::Ace,
            ItalianRank::Two,
            ItalianRank::Three,
            ItalianRank::Four,
            ItalianRank::Five,
            ItalianRank::Six,
            ItalianRank::Seven,
            ItalianRank::Jack,
            ItalianRank::Knight,
            ItalianRank::King,
        ]
        .iter()
        .flat_map(|&rank| {
            [Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades]
                .iter()
                .map(move |&suit| ItalianCard::new(rank, suit))
        })
        .collect();

        proptest::sample::subsequence(all_cards, size).prop_map(Deck::from)
    }
}
