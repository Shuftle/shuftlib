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

pub mod cards;
pub mod engine;
pub mod tressette;
