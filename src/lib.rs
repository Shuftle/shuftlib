// Allow panics, unwraps, and expects in test code
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::panic_in_result_fn
    )
)]
#![doc = include_str!("../README.md")]

/// Core primitives for card games (cards, decks, game mechanics).
pub mod core;
/// Contains the logic for the tressette game.
pub mod tressette;
/// Trick-taking game mechanics available at the crate root.
pub mod trick_taking;
