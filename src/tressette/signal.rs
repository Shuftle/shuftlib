//! Signals of Tressette: what the seat leading a trick may say about the suit
//! it leads, such as "volo" for its last card of that suit.
//!
//! Signals are never bluffs: a [`Lead`] only exists when the leader's hand
//! matches its signal.

use super::{Error, TressetteCard};

/// A signal the seat leading a trick may give with its card.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Signal {
    /// The card led is the leader's last of its suit.
    Volo,
}

/// A card led with a signal the leader's hand matches. Only [`Lead::parse`]
/// makes one, so holding one means the signal is honest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Lead {
    card: TressetteCard,
    signal: Signal,
}

impl Lead {
    /// Reads `signal` on `card` against `rest`, the cards the leader holds
    /// besides it, or says why the signal is false.
    pub(super) fn parse(
        card: TressetteCard,
        signal: Signal,
        rest: &[TressetteCard],
    ) -> Result<Self, Error> {
        let matches = match signal {
            Signal::Volo => rest.iter().all(|held| held.suit() != card.suit()),
        };
        if matches {
            Ok(Self { card, signal })
        } else {
            Err(Error::FalseSignal { signal })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::italian::ItalianRank;
    use crate::tressette::{
        card::test_utils::{cards_strategy, tressette_card_strategy},
        deck,
    };
    use proptest::prelude::*;

    /// Returns `same_suit` cards of `lead`'s suit other than `lead`, and the
    /// cards of `other_suits` not of that suit.
    fn hand(
        lead: TressetteCard,
        same_suit: usize,
        other_suits: &[TressetteCard],
    ) -> Vec<TressetteCard> {
        let suit = deck()
            .into_iter()
            .filter(|card| card.suit() == lead.suit() && *card != lead)
            .take(same_suit);
        let others = other_suits
            .iter()
            .copied()
            .filter(|card| card.suit() != lead.suit());
        suit.chain(others).collect()
    }

    proptest! {
        /// Volo is true exactly when the leader holds no other card of the
        /// suit led.
        #[test]
        fn volo_is_true_exactly_when_the_card_is_the_last_of_its_suit(
            lead in tressette_card_strategy(),
            same_suit in 0..ItalianRank::ALL.len(),
            other_suits in cards_strategy(),
        ) {
            let rest = hand(lead, same_suit, &other_suits);
            prop_assert_eq!(Lead::parse(lead, Signal::Volo, &rest).is_ok(), same_suit == 0);
        }
    }
}
