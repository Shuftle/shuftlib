use std::{fmt::Display, ops::Deref};

use super::{PLAYERS, PlayerId, TrickTakingGame};

/// A trick is a set containing the cards played and the player who won the
/// trick, represented as `PlayerId`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Trick<G>
where
    G: TrickTakingGame,
{
    cards: [G::CardType; PLAYERS],
    taker: PlayerId,
}

impl<G> Display for Trick<G>
where
    G: TrickTakingGame,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.cards[0], self.cards[1], self.cards[2], self.cards[3], self.taker
        )
    }
}

impl<G> Trick<G>
where
    G: TrickTakingGame,
{
    /// Creates a new Trick from cards and the player who won.
    pub fn new(cards: [G::CardType; PLAYERS], taker: PlayerId) -> Self {
        Self { cards, taker }
    }

    /// Returns the card this trick has been won with.
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::trick_taking::{Trick, PlayerId};
    /// use shuftlib::core::Suit;
    /// use shuftlib::core::italian::ItalianRank;
    /// use shuftlib::tressette::{TressetteRules, TressetteCard};
    ///
    /// let cards = [
    ///     TressetteCard::new(ItalianRank::Ace, Suit::Hearts),
    ///     TressetteCard::new(ItalianRank::Two, Suit::Hearts),
    ///     TressetteCard::new(ItalianRank::Three, Suit::Hearts),
    ///     TressetteCard::new(ItalianRank::Four, Suit::Hearts),
    /// ];
    /// let trick = Trick::<TressetteRules>::new(cards, PlayerId::PLAYER_0);
    /// assert_eq!(trick.taken_with(), cards[0]);
    /// ```
    pub fn taken_with(&self) -> G::CardType {
        self.cards[self.taker.as_usize()]
    }

    /// Returns the `PlayerId` of the player who won the trick.
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::trick_taking::{Trick, PlayerId};
    /// use shuftlib::core::Suit;
    /// use shuftlib::core::italian::ItalianRank;
    /// use shuftlib::tressette::{TressetteRules, TressetteCard};
    ///
    /// let cards = [
    ///     TressetteCard::new(ItalianRank::Ace, Suit::Hearts),
    ///     TressetteCard::new(ItalianRank::Two, Suit::Hearts),
    ///     TressetteCard::new(ItalianRank::Three, Suit::Hearts),
    ///     TressetteCard::new(ItalianRank::Four, Suit::Hearts),
    /// ];
    /// let trick = Trick::<TressetteRules>::new(cards, PlayerId::PLAYER_0);
    /// assert_eq!(trick.taker(), PlayerId::PLAYER_0);
    /// ```
    pub fn taker(&self) -> PlayerId {
        self.taker
    }

    /// Returns the cards played during this trick.
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::trick_taking::{Trick, PlayerId};
    /// use shuftlib::core::Suit;
    /// use shuftlib::core::italian::ItalianRank;
    /// use shuftlib::tressette::{TressetteRules, TressetteCard};
    ///
    /// let cards = [
    ///     TressetteCard::new(ItalianRank::Ace, Suit::Hearts),
    ///     TressetteCard::new(ItalianRank::Two, Suit::Hearts),
    ///     TressetteCard::new(ItalianRank::Three, Suit::Hearts),
    ///     TressetteCard::new(ItalianRank::Four, Suit::Hearts),
    /// ];
    /// let trick = Trick::<TressetteRules>::new(cards, PlayerId::PLAYER_0);
    /// assert_eq!(trick.cards(), &cards);
    /// ```
    pub fn cards(&self) -> &[G::CardType] {
        &self.cards
    }
}

/// A temporary state of a trick that's still not over: not all the players made
/// their move or a taker hasn't been determined yet.
#[derive(Clone, Copy, Debug)]
pub struct OngoingTrick<G>
where
    G: TrickTakingGame,
{
    cards: [Option<G::CardType>; PLAYERS],
    first_to_play: PlayerId,
    next_to_play: PlayerId,
    play_count: usize,
}

impl<G> Deref for OngoingTrick<G>
where
    G: TrickTakingGame,
{
    type Target = [Option<G::CardType>; PLAYERS];

    fn deref(&self) -> &Self::Target {
        &self.cards
    }
}

impl<G> OngoingTrick<G>
where
    G: TrickTakingGame,
{
    /// Adds the `Card` passed as parameter to the `OngoingTrick`.
    /// Checking the validity of the card played is a responsibility of the
    /// caller.
    ///
    /// # Examples
    /// ```
    /// use shuftlib::trick_taking::{OngoingTrick, PlayerId, TrickTakingGame};
    /// use shuftlib::core::{Card, Suit};
    /// use shuftlib::core::italian::ItalianRank;
    /// use shuftlib::tressette::{TressetteRules, TressetteCard};
    ///
    /// let first_to_play = PlayerId::PLAYER_0;
    /// let card = TressetteCard::new(ItalianRank::Ace, Suit::Hearts);
    /// let mut trick = OngoingTrick::<TressetteRules>::new(first_to_play);
    /// trick.play(card);
    /// let mut second_to_play = first_to_play;
    /// second_to_play.inc();
    ///
    /// assert_eq!(trick[0], Some(card));
    /// assert_eq!(trick.next_to_play(), second_to_play)
    /// ```
    pub fn play(&mut self, card: G::CardType) {
        self.cards[self.next_to_play.as_usize()] = Some(card);
        self.next_to_play.inc();
        self.play_count += 1;
    }

    /// Tries to transform the current `OngoingTrick` into a `Trick` by
    /// determining the taker of the trick. It doesn't make any assumption on
    /// previously played cards during the current `OngoingHand`. It also does
    /// not check if it contains duplicates since that could be valid in some games.
    ///
    /// # Errors
    ///
    /// Fails if any of the moves of the `OngoingTrick` this is called on is
    /// None. It means that not all players made their move yet, so a taker
    /// can't be determined.
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::trick_taking::{OngoingTrick, PlayerId, TrickTakingGame};
    /// use shuftlib::core::Suit;
    /// use shuftlib::core::italian::ItalianRank;
    /// use shuftlib::tressette::{TressetteRules, TressetteCard};
    ///
    /// let cards = [
    ///   TressetteCard::new(ItalianRank::Ace, Suit::Hearts),
    ///   TressetteCard::new(ItalianRank::Two, Suit::Hearts),
    ///   TressetteCard::new(ItalianRank::Three, Suit::Hearts),
    ///   TressetteCard::new(ItalianRank::Four, Suit::Hearts),
    /// ];
    /// let first_to_play = PlayerId::PLAYER_0;
    /// let mut ongoing_trick = OngoingTrick::<TressetteRules>::new(first_to_play);
    /// ongoing_trick.play(cards[0]);
    ///
    /// // After only playing a card, it's not possible to finish the OngoingTrick.
    /// assert!(ongoing_trick.clone().finish().is_none());
    ///
    /// let mut to_play = first_to_play;
    /// to_play.inc();
    /// cards.iter().skip(1).for_each(|&c| {
    ///   ongoing_trick.play(c);
    ///   to_play.inc();
    /// });
    ///
    /// // After every player made their play, it's possible to get the trick.
    /// let trick = ongoing_trick.finish().unwrap();
    /// // Finishing the trick also means determining a taker. Since in this
    /// // example we are using the tressette game rules, player 2 is the taker.
    /// assert_eq!(trick.taker(), PlayerId::PLAYER_2);
    /// ```
    pub fn finish(self) -> Option<Trick<G>> {
        let mut cards: [G::CardType; PLAYERS] = [G::CardType::default(); PLAYERS];
        if self
            .iter()
            .enumerate()
            .map(|(i, &x)| {
                if let Some(c) = x {
                    cards[i] = c;
                    true
                } else {
                    false
                }
            })
            .any(|is_some| !is_some)
        {
            return None;
        }

        let taker = G::determine_taker(&cards, self.first_to_play);
        Some(Trick { cards, taker })
    }

    /// Returns the cards contained in this `OngoingTrick`.
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::core::trick_taking::{OngoingTrick, PlayerId};
    /// use shuftlib::tressette::TressetteRules;
    ///
    /// let trick = OngoingTrick::<TressetteRules>::new(PlayerId::PLAYER_0);
    /// assert!(trick.cards().iter().all(|c| c.is_none()));
    /// ```
    pub fn cards(&self) -> &[Option<G::CardType>] {
        &self.cards
    }

    /// Returns the id of the person who starts the trick.
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::core::trick_taking::{OngoingTrick, PlayerId};
    /// use shuftlib::tressette::TressetteRules;
    ///
    /// let trick = OngoingTrick::<TressetteRules>::new(PlayerId::PLAYER_0);
    /// assert_eq!(trick.first_to_play(), PlayerId::PLAYER_0);
    /// ```
    pub fn first_to_play(&self) -> PlayerId {
        self.first_to_play
    }

    /// Returns the id of the person who plays next in the trick.
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::core::trick_taking::{OngoingTrick, PlayerId};
    /// use shuftlib::tressette::TressetteRules;
    ///
    /// let trick = OngoingTrick::<TressetteRules>::new(PlayerId::PLAYER_0);
    /// assert_eq!(trick.next_to_play(), PlayerId::PLAYER_0);
    /// ```
    pub fn next_to_play(&self) -> PlayerId {
        self.next_to_play
    }

    /// Creates a new `OngoingTrick` with all card slots empty, starting with
    /// the specified player.
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::core::trick_taking::{OngoingTrick, PlayerId, TrickTakingGame};
    /// use shuftlib::tressette::TressetteRules;
    ///
    /// let first_to_play = PlayerId::PLAYER_0;
    /// let ongoing_trick = OngoingTrick::<TressetteRules>::new(first_to_play);
    ///
    /// assert_eq!(ongoing_trick.first_to_play(), first_to_play);
    /// ongoing_trick.cards().iter().for_each(|&c| assert!(c.is_none()));
    /// ```
    pub fn new(first_to_play: PlayerId) -> Self {
        Self {
            cards: [None; PLAYERS],
            first_to_play,
            next_to_play: first_to_play,
            play_count: 0,
        }
    }
}

/// Testing utilities for trick-taking game components.
#[cfg(test)]
pub mod test_utils {
    use proptest::collection::hash_set;
    use proptest::prelude::*;

    use crate::core::deck::Deck;
    pub use crate::core::italian::test_utils::italian_card_strategy;
    use crate::trick_taking::{Hand, OngoingTrick, PlayerId, TrickTakingGame};
    use crate::{core::italian::ItalianCard, trick_taking::PLAYERS};

    /// A minimal test implementation of TrickTakingGame for testing purposes.
    ///
    /// This implementation always determines PLAYER_0 as the trick taker.
    #[derive(Clone, Copy, Debug)]
    pub struct TestGame {}

    impl TrickTakingGame for TestGame {
        type CardType = ItalianCard;

        fn determine_taker(
            _cards: &[Self::CardType; PLAYERS],
            _first_to_play: PlayerId,
        ) -> super::super::PlayerId {
            PlayerId::PLAYER_0
        }

        fn deck() -> Deck<Self::CardType> {
            Deck::italian()
        }

        fn score_hand(_hand: &Hand<Self>) -> (u8, u8)
        where
            Self: Sized,
        {
            (0, 0)
        }

        fn is_game_over(_scores: (u8, u8)) -> bool {
            true
        }
    }

    /// Strategy to create an `OngoingTrick` filled with random cards.
    pub fn ongoing_trick_strategy() -> impl Strategy<Value = OngoingTrick<TestGame>> {
        hash_set(italian_card_strategy(), PLAYERS).prop_map(|hash_set| {
            let mut cards = [None; PLAYERS];
            hash_set
                .iter()
                .enumerate()
                .for_each(|(i, &c)| cards[i] = Some(c));

            OngoingTrick {
                cards,
                first_to_play: PlayerId::PLAYER_0,
                next_to_play: PlayerId::PLAYER_0,
                play_count: 0,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use proptest::{array, prelude::*};

    use super::test_utils::{TestGame, italian_card_strategy, ongoing_trick_strategy};
    use super::{OngoingTrick, PlayerId};

    proptest! {
        #[test]
        fn play_method_works(cards in array::uniform4(italian_card_strategy())) {
            let mut trick: OngoingTrick<TestGame> = OngoingTrick::new(PlayerId::PLAYER_0);

            for (index, &card) in cards.iter().enumerate() {
                trick.play(card);
                assert_eq!(trick[index], Some(card));
            }
        }

        #[test]
        fn finish_method_works(ongoing_trick in ongoing_trick_strategy()) {
            let trick = ongoing_trick.finish().ok_or_else(|| TestCaseError::fail("trick should be complete"))?;

            let cards = ongoing_trick.cards();

            prop_assert_eq!(trick.taker(), PlayerId::PLAYER_0);
            prop_assert_eq!(trick.taken_with(), cards[0].ok_or_else(|| TestCaseError::fail("card should exist"))?);
        }
    }
}
