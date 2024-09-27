//! This crate contains all the necessary types, methods, functions and traits
//! to work with cards, decks and card games.
#![expect(incomplete_features)]
#![feature(generic_const_exprs)]

/// Contains basic types common to various card games.
pub mod common;
/// Contains the logic relative to the tressette engine.
pub mod tressette;
