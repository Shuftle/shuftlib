//! A hand: the sequence of tricks played between one deal and the next.
//!
//! [`OngoingHand`] accumulates tricks as they're completed; [`OngoingHand::finish`]
//! produces an immutable [`Hand`] once all tricks are done.

use crate::trick_taking::{OngoingTrick, PlayerId, TRICKS, Trick, TrickTakingGame};

/// Various games are usually played multiple times, until one team reaches a
/// certain score. These "multiple times" are called hands: "We played a game of
/// tressette and our team won in just 2 hands!".
///
/// This type is generic over the actual card type and the number of tricks it
/// takes to finish the hand.
#[derive(Debug, Clone, Copy)]
pub struct Hand<G>
where
    G: TrickTakingGame,
{
    tricks: [Trick<G>; TRICKS],
}

impl<G> Hand<G>
where
    G: TrickTakingGame,
{
    /// Creates a new Hand from an array of tricks.
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::{trick_taking::{Hand, Trick, PlayerId}, core::{Suit, italian::ItalianRank}, tressette::{TressetteRules, TressetteCard}};
    ///
    /// let cards = [
    ///     TressetteCard::new(ItalianRank::Ace, Suit::Hearts),
    ///     TressetteCard::new(ItalianRank::Two, Suit::Hearts),
    ///     TressetteCard::new(ItalianRank::Three, Suit::Hearts),
    ///     TressetteCard::new(ItalianRank::Four, Suit::Hearts),
    /// ];
    /// let trick = Trick::<TressetteRules>::new(cards, PlayerId::PLAYER_0);
    /// let tricks = std::array::from_fn(|_| trick.clone());
    /// let hand = Hand::new(tricks);
    /// ```
    pub fn new(tricks: [Trick<G>; TRICKS]) -> Self {
        Self { tricks }
    }

    /// Returns a reference to the tricks of this [`Hand<G>`].
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::{trick_taking::{Hand, Trick, PlayerId}, core::{Suit, italian::ItalianRank}, tressette::{TressetteRules, TressetteCard}};
    ///
    /// let cards = [
    ///     TressetteCard::new(ItalianRank::Ace, Suit::Hearts),
    ///     TressetteCard::new(ItalianRank::Two, Suit::Hearts),
    ///     TressetteCard::new(ItalianRank::Three, Suit::Hearts),
    ///     TressetteCard::new(ItalianRank::Four, Suit::Hearts),
    /// ];
    /// let trick = Trick::<TressetteRules>::new(cards, PlayerId::PLAYER_0);
    /// let tricks = std::array::from_fn(|_| trick.clone());
    /// let hand = Hand::new(tricks);
    /// assert_eq!(hand.tricks().len(), 10);
    /// ```
    pub fn tricks(&self) -> &[Trick<G>; TRICKS] {
        &self.tricks
    }
}

/// A hand takes multiple turns for each player to be completed, this is the
/// representation of a `Hand` which hasn't been completed yet.
#[derive(Clone, Copy, Debug)]
pub struct OngoingHand<G>
where
    G: TrickTakingGame,
{
    current_trick: OngoingTrick<G>,
    tricks: [Option<Trick<G>>; TRICKS],
}

impl<G> OngoingHand<G>
where
    G: TrickTakingGame,
{
    /// Returns the current trick of this [`OngoingHand<G>`].
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::{trick_taking::{OngoingHand, PlayerId}, tressette::TressetteRules};
    ///
    /// let hand = OngoingHand::<TressetteRules>::new(PlayerId::PLAYER_0);
    /// assert_eq!(hand.current_trick().first_to_play(), PlayerId::PLAYER_0);
    /// ```
    pub fn current_trick(&self) -> &OngoingTrick<G> {
        &self.current_trick
    }

    /// Returns a reference to the tricks of this [`OngoingHand<G>`].
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::{trick_taking::{OngoingHand, PlayerId}, tressette::TressetteRules};
    ///
    /// let hand = OngoingHand::<TressetteRules>::new(PlayerId::PLAYER_0);
    /// assert!(hand.tricks().iter().all(|t| t.is_none()));
    /// ```
    pub fn tricks(&self) -> &[Option<Trick<G>>; TRICKS] {
        &self.tricks
    }

    /// Transforms an `OngoingHand` into a `Hand`, a read-only data structure
    /// used to just store the information related to a hand that has been played.
    ///
    /// Returns `None` if not all tricks have been completed.
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::{trick_taking::{OngoingHand, Hand, Trick, PlayerId}, core::{Suit, italian::ItalianRank}, tressette::{TressetteRules, TressetteCard}};
    ///
    /// let ongoing_hand = OngoingHand::<TressetteRules>::new(PlayerId::PLAYER_0);
    /// assert!(ongoing_hand.finish().is_none());
    /// ```
    pub fn finish(self) -> Option<Hand<G>> {
        let tricks: [Trick<G>; TRICKS] = self
            .tricks
            .into_iter()
            .collect::<Option<Vec<_>>>()?
            .try_into()
            .ok()?;
        Some(Hand { tricks })
    }

    /// Constructor for `OngoingHand`. Starts a fresh, empty trick led by
    /// `first_to_play`; `tricks` are all initialized empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::{trick_taking::{OngoingHand, PlayerId}, tressette::TressetteRules};
    ///
    /// let ongoing_hand = OngoingHand::<TressetteRules>::new(PlayerId::PLAYER_0);
    ///
    /// assert_eq!(ongoing_hand.current_trick().first_to_play(), PlayerId::PLAYER_0);
    /// ongoing_hand.tricks().iter().for_each(|t| assert!(t.is_none()));
    /// ```
    pub fn new(first_to_play: PlayerId) -> Self {
        Self {
            tricks: [const { None }; TRICKS],
            current_trick: OngoingTrick::new(first_to_play),
        }
    }

    /// Adds a trick to this hand.
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::{trick_taking::{OngoingHand, Trick, PlayerId}, core::{Suit, italian::ItalianRank}, tressette::{TressetteRules, TressetteCard}};
    ///
    /// let mut hand = OngoingHand::<TressetteRules>::new(PlayerId::PLAYER_0);
    /// let cards = [
    ///     TressetteCard::new(ItalianRank::Ace, Suit::Hearts),
    ///     TressetteCard::new(ItalianRank::Two, Suit::Hearts),
    ///     TressetteCard::new(ItalianRank::Three, Suit::Hearts),
    ///     TressetteCard::new(ItalianRank::Four, Suit::Hearts),
    /// ];
    /// let trick = Trick::<TressetteRules>::new(cards, PlayerId::PLAYER_0);
    /// hand.add(trick, 0);
    /// ```
    pub fn add(&mut self, trick: Trick<G>, id: usize) {
        self.tricks[id] = Some(trick);
    }

    /// Returns a mutable reference to the current trick of this [`OngoingHand<G>`].
    pub fn current_trick_mut(&mut self) -> &mut OngoingTrick<G> {
        &mut self.current_trick
    }

    /// Sets the current trick of this [`OngoingHand<G>`].
    pub fn set_current_trick(&mut self, trick: OngoingTrick<G>) {
        self.current_trick = trick;
    }
}
