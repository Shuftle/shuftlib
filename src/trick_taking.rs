//! Trick-taking game mechanics including players, tricks, and hands.
//!
//! This module contains the fundamental building blocks for trick-taking card games:
//! - Generic player management and identification
//! - Trick and ongoing trick state management
//! - Hand completion and scoring mechanics
//! - Core trait for implementing different trick-taking games

use crate::core::Card;

/// Number of tricks in all games.
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
    fn determine_taker(cards: &[Self::CardType; PLAYERS], first_to_play: PlayerId) -> PlayerId;

    /// Calculate the score for a completed hand.
    ///
    /// Returns the updated scores for both teams: (team_0_2_score, team_1_3_score).
    /// Default implementation returns unchanged scores.
    fn score_hand(&self, _hand: &Hand<Self>) -> (u8, u8)
    where
        Self: Sized,
    {
        (0, 0)
    }

    /// Check if the game is over based on current scores.
    ///
    /// Returns true if the game should end with the given scores.
    /// Default implementation never ends the game.
    fn is_game_over(&self, _scores: (u8, u8)) -> bool {
        false
    }

    /// Get the number of cards dealt to each player at the start of a hand.
    ///
    /// Default is 10 cards (standard for Tressette).
    fn hand_size(&self) -> usize {
        10
    }
}

/// Generic player management and identification.
pub mod player;
pub use player::{PLAYERS, Player, PlayerId};

/// Trick and ongoing trick state management.
pub mod trick;
pub use trick::{OngoingTrick, Trick};

/// Hand completion and scoring mechanics.
pub mod hand;
pub use hand::{Hand, OngoingHand};
