use crate::common::{cards::{Card, ItalianCard, ItalianRank, Suit}, hands::{PlayerId, TrickTakingGame}};
use std::{fmt::Display, ops::Deref};

#[derive(Clone, Debug, Default)]
/// Contains the rules of the briscola game.
pub struct BriscolaRules{}

impl TrickTakingGame for BriscolaRules {
    type CardType = BriscolaCard;

    const PLAYERS: usize = 4;

    const TRICKS: usize = 10;

    /// Determines who won the trick, by the rules of Briscola.
    /// The winner of the trick is always the player who played
    /// the highest card with the same `Suit` of the first `BriscolaCard`
    /// played that trick, unless a briscola was played. If that's the case, the highest briscola card wins. See the implementation of `Ord` and `PartialOrd` for
    /// `BriscolaCard` for more info. The implementation of this trait is meant
    /// to only be used internally by `OngoingTrick`, however it's possible to
    /// call it elsewhere if needed. It also assumes the slice `cards` is valid
    /// for the briscola game, so it assumes there are no duplicates. It's a
    /// responsability of the caller to make sure that's the case.
    /// ```
    /// #![feature(generic_const_exprs)]
    /// use shuftlib::common::{hands::{TrickTakingGame, PlayerId}, cards::{ItalianRank, Suit}};
    /// use shuftlib::briscola::{BriscolaRules, BriscolaCard};
    ///
    /// let cards = [
    ///   BriscolaCard::new(ItalianRank::Ace, Suit::Hearts),
    ///   BriscolaCard::new(ItalianRank::Two, Suit::Hearts),
    ///   BriscolaCard::new(ItalianRank::Three, Suit::Hearts),
    ///   BriscolaCard::new(ItalianRank::Four, Suit::Hearts),
    /// ];
    ///
    /// let taker = TressetteRules::determine_taker(&cards, PlayerId::new(2).unwrap());
    /// assert_eq!(taker, PlayerId::new(2).unwrap());
    /// ```
    fn determine_taker(
        cards: &[Self::CardType; Self::PLAYERS],
        first_to_play: PlayerId<{ Self::PLAYERS }>,
    ) -> PlayerId<{ Self::PLAYERS }> {
        PlayerId::new(0).unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy, Default)]
/// Representation of a card used in the Briscola game. It's just a new type
/// over `ItalianCard`.
pub struct BriscolaCard {
    card: ItalianCard
}

impl Card for BriscolaCard {}

impl Display for BriscolaCard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.card)
    }
}

impl From<ItalianCard> for BriscolaCard {
    fn from(value: ItalianCard) -> Self {
        BriscolaCard { card: value }
    }
}

impl Deref for BriscolaCard {
    type Target = ItalianCard;

    fn deref(&self) -> &Self::Target {
        &self.card
    }
}

impl BriscolaCard{
    /// Gets the value of the card by the rules of the Briscola game:
    /// - Ace = 11
    /// - 3 = 10
    /// - King = 4
    /// - Knight = 3
    /// - Jack = 2
    /// - The rest = 0
    /// 
    /// # Examples
    /// ```
    /// #![feature(generic_const_exprs)]
    /// use shuftlib::{briscola::BriscolaCard, common::cards::{Suit, ItalianRank}};
    /// use num_rational::Rational32;
    /// 
    /// let ace = BriscolaCard::new(ItalianRank::Ace, Suit::Hearts);
    /// let three = BriscolaCard::new(ItalianRank::Three, Suit::Spades);
    /// let four = BriscolaCard::new(ItalianRank::Four, Suit::Clubs);
    /// assert_eq!(ace.value(), 11);
    /// assert_eq!(three.value(), 10);
    /// assert_eq!(four.value(), 0);
    /// ```
    pub fn value(&self) -> u8 {
        match self.rank() {
            ItalianRank::Ace => 11,
            ItalianRank::Three => 10,
            ItalianRank::King => 4,
            ItalianRank::Knight => 3,
            ItalianRank::Jack => 2,
            _ => 0,
        }
    }

    /// Generates a new `BriscolaCard` starting from an `ItalianRank` and
    /// a `Suit`.
    /// ```
    /// #![feature(generic_const_exprs)]
    /// use shuftlib::common::cards::{ItalianCard, ItalianRank, Suit};
    /// use shuftlib::briscola::BriscolaCard;
    ///
    /// let suit = Suit::Spades;
    /// let rank = ItalianRank::Ace;
    /// assert_eq!(*BriscolaCard::new(rank, suit), ItalianCard::new(rank, suit));
    /// ```
    pub fn new(rank: ItalianRank, suit: Suit) -> Self {
        BriscolaCard { card: ItalianCard::new(rank, suit) }
    }
}

#[cfg(test)]
mod tests {
    use proptest::{prelude::*, prop_oneof};

    use super::*;

    fn briscola_card_strategy() -> impl Strategy<Value = BriscolaCard> {
        (
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
            ],
            prop_oneof![
                Just(Suit::Hearts),
                Just(Suit::Clubs),
                Just(Suit::Spades),
                Just(Suit::Diamonds),
            ],
        )
            .prop_map(|(rank, suit)| BriscolaCard::new(rank, suit))
    }
    proptest! {
        #[test]
        fn test_value(card in briscola_card_strategy()) {
            let expected_value = match card.rank() {
                ItalianRank::Ace => 11,
                ItalianRank::Three => 10,
                ItalianRank::King => 4,
                ItalianRank::Knight => 3,
                ItalianRank::Jack => 2,
                _ => 0,
            };
            assert_eq!(card.value(), expected_value);
        }}
    }
