//! Tressette card game implementation.
//!
//! This module contains the core types and logic for playing tressette,
//! including card representations, game rules, position management, and
//! high-level game state.

/// Card representation for tressette.
pub mod card;

/// Game rules implementation.
pub mod rules;

/// High-level game state management for move-based gameplay.
pub mod game;

// Re-export main types for convenience
pub use card::TressetteCard;
pub use rules::{TressetteRules, SCORE_TO_WIN};
pub use game::{Game, MoveEffect, Status, Error};
