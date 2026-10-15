//! Trick-taking game mechanics including players, tricks, and hands.
//!
//! This module contains the fundamental building blocks for trick-taking card games:
//! - Generic player management and identification
//! - Trick and ongoing trick state management
//! - Hand completion and scoring mechanics
//! - Core trait for implementing different trick-taking games

use crate::cards::{Card, deck::Deck};

/// Number of tricks in a hand.
pub const TRICKS: usize = 10;

/// Trait for trick-taking games.
///
/// This trait provides a foundation for implementing different trick-taking card games.
pub trait TrickTakingGame {
    /// Define the type of card that's going to be used in this game.
    type CardType: Card;

    /// Every trick taking game has some logic to determine the winner (or
    /// taker) of the trick. The taker is generally determined by the cards that
    /// have been played and it can depend by the order in which the players
    /// played their cards.
    fn determine_taker(cards: &[Self::CardType; PLAYERS], first_to_play: Seat) -> Seat;

    /// Calculate the score for a completed hand.
    ///
    /// Returns the updated scores for both teams: (North-South, East-West).
    fn score_hand(hand: &Hand<Self>) -> (u8, u8)
    where
        Self: Sized;

    /// Check if the game is over based on current scores.
    ///
    /// Returns true if the game should end with the given scores.
    fn is_game_over(scores: (u8, u8)) -> bool;

    /// Get a deck as per the rules of the game.
    fn deck() -> Deck<Self::CardType>;
}

/// Generic player management and identification.
pub mod player;
pub use player::{PLAYERS, Player, Seat, Team};

/// Trick and ongoing trick state management.
pub mod trick;
pub use trick::{OngoingTrick, Trick};

/// Hand completion and scoring mechanics.
pub mod hand;
pub use hand::{Hand, OngoingHand};
