use std::{
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
};

use rand::Rng;
use strum::{EnumIter, FromRepr, IntoEnumIterator};

/// A trait representing a card. The actual implementation depends on the game where this is used.
pub trait Card: Display + Default + Sized + Debug + Copy + Eq + PartialEq {}

/// Representation of a card that goes into an Italian deck.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ItalianCard {
    rank: ItalianRank,
    suit: Suit,
}

impl ItalianCard {
    /// Generates a card with the given rank and suit
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Representation of a card that goes into an French deck.
pub struct FrenchCard {
    rank: FrenchRank,
    suit: Suit,
}

impl FrenchCard {
    /// Generates a card with the given rank and suit
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
            FrenchWithJoker::Normal(c) => write!(f, "{}", c),
            FrenchWithJoker::Joker(c) => write!(f, "{}", c),
        }
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, Hash)]
/// The 4 suits of a standard deck. They have an equivalent in pretty much all regional decks.
/// In some games they have a hierarchical order.
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
        write!(f, "{}", s)
    }
}

#[derive(Default)]
/// Represents a deck of cards. Cards can be added or removed at will.
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
    pub fn italian() -> Deck<ItalianCard> {
        let mut cards = Vec::with_capacity(ITALIAN_CARDS);
        for suit in Suit::iter() {
            for rank in ItalianRank::iter() {
                cards.push(ItalianCard { rank, suit });
            }
        }

        Deck { cards }
    }
}

impl Deck<FrenchCard> {
    /// Creates a new 52 cards French deck.
    pub fn french() -> Deck<FrenchCard> {
        let mut cards = Vec::with_capacity(FRENCH_CARDS);
        for suit in Suit::iter() {
            for rank in FrenchRank::iter() {
                cards.push(FrenchCard { suit, rank });
            }
        }

        Deck { cards }
    }

    /// Creates a new 52 cards French deck, with the addition of the specified amount of jokers.
    pub fn french_with_jokers(jokers: u8) -> Deck<FrenchWithJoker> {
        let mut cards = Vec::with_capacity(FRENCH_CARDS + jokers as usize);
        for suit in Suit::iter() {
            for rank in FrenchRank::iter() {
                cards.push(FrenchWithJoker::Normal(FrenchCard { suit, rank }));
            }
        }

        for _ in 0..jokers {
            cards.push(FrenchWithJoker::Joker(Joker {}));
        }

        Deck { cards }
    }
}

impl<T: Card> Deck<T> {
    /// Performs a random permutation on the deck with the Fisher–Yates shuffle algorithm, repeated 10 times.
    pub fn shuffle(&mut self) {
        let mut rng = rand::thread_rng();
        let max = self.cards.len();
        for _ in 0..10 {
            for i in 0..max - 2 {
                let j = rng.gen_range(i..max);
                self.cards.swap(i, j);
            }
        }
    }

    /// Adds a card in a random position inside the deck.
    pub fn shuffle_card(&mut self, card: T) {
        let mut rng = rand::thread_rng();
        let max = self.cards.len();
        let position = rng.gen_range(1..max);
        self.cards.insert(position, card);
    }

    /// Adds a card to the top of the deck.
    pub fn push(&mut self, card: T) {
        self.cards.push(card);
    }

    /// Draws the top-most card in the deck. It returns None if there are no cards left.
    pub fn draw(&mut self) -> Option<T> {
        self.cards.pop()
    }

    /// Creates a new empty deck.
    pub fn new() -> Deck<T> {
        Deck { cards: Vec::new() }
    }

    /// Creates a new empty deck with specified capacity.
    pub fn with_capacity(capacity: usize) -> Deck<T> {
        Deck {
            cards: Vec::with_capacity(capacity),
        }
    }

    /// Creates a new deck from a given vec of cards.
    pub fn from_vec(cards: Vec<T>) -> Deck<T> {
        Deck { cards }
    }

    /// Returns the number of cards left in the deck.
    pub fn len(&self) -> usize {
        self.cards.len()
    }

    /// Returns whether or not the deck is empty.
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
}

impl<T> Deref for Deck<T>
where
    T: Card,
{
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.cards
    }
}

impl<T> DerefMut for Deck<T>
where
    T: Card,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cards
    }
}

#[cfg(test)]
mod tests {
    use crate::common::cards::Deck;

    #[test]
    fn should_shuffle() {
        let sorted_deck = Deck::italian();

        let mut shuffled_deck = Deck::italian();
        shuffled_deck.shuffle();

        let decks = sorted_deck.cards.iter().zip(shuffled_deck.cards.iter());

        let count_of_different_cards = decks.filter(|(&card1, &card2)| card1 != card2).count();

        assert_ne!(count_of_different_cards, 0);
    }
}
