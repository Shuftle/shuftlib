//! Signals of Tressette: what the seat leading a trick may say about the suit
//! it leads, such as "volo" for its last card of that suit.
//!
//! Signals are never bluffs: a [`Lead`] only exists when the leader's hand
//! matches its signal.

use crate::cards::italian::ItalianRank;

use super::{Error, TressetteCard};

/// A signal the seat leading a trick may give with its card.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Signal {
    /// The card led is the leader's last of its suit.
    Volo,
    /// The leader holds exactly one other card of the suit, King or lower.
    Altra,
}

/// The card that opens a trick, with the signal given alongside it. It can
/// only be built from a hand that matches the signal, so every `Lead` is
/// honest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Lead {
    card: TressetteCard,
    signal: Signal,
}

impl Lead {
    /// Reads `signal` on `card` against `hand`, the leader's cards before it
    /// leads, or says why the signal is false.
    pub(super) fn parse(
        card: TressetteCard,
        signal: Signal,
        hand: &[TressetteCard],
    ) -> Result<Self, Error> {
        let mut same_suit = hand
            .iter()
            .filter(|held| held.suit() == card.suit() && **held != card);
        let matches = match signal {
            Signal::Volo => same_suit.next().is_none(),
            Signal::Altra => {
                let ace = TressetteCard::new(ItalianRank::Ace, card.suit());
                matches!(
                    (same_suit.next(), same_suit.next()),
                    (Some(other), None) if *other < ace
                )
            }
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
    use crate::tressette::test_utils::hand_strategy;
    use proptest::{prelude::*, sample::Index};

    /// Returns the cards of `hand` in `lead`'s suit, other than `lead`.
    fn same_suit(hand: &[TressetteCard], lead: TressetteCard) -> Vec<TressetteCard> {
        hand.iter()
            .copied()
            .filter(|card| card.suit() == lead.suit() && *card != lead)
            .collect()
    }

    proptest! {
        /// Volo is true exactly when the leader holds no other card of the
        /// suit led.
        #[test]
        fn volo_is_true_exactly_when_the_card_is_the_last_of_its_suit(
            hand in hand_strategy(),
            pick in any::<Index>(),
        ) {
            let lead = *pick.get(&hand);
            let expected = same_suit(&hand, lead).is_empty();
            prop_assert_eq!(Lead::parse(lead, Signal::Volo, &hand).is_ok(), expected);
        }

        /// Altra is true exactly when the leader holds one other card of the
        /// suit led, and it's King or lower.
        #[test]
        fn altra_is_true_exactly_when_one_low_card_of_the_suit_is_left(
            hand in hand_strategy(),
            pick in any::<Index>(),
        ) {
            let lead = *pick.get(&hand);
            let same_suit = same_suit(&hand, lead);
            let ace = TressetteCard::new(ItalianRank::Ace, lead.suit());
            let expected = same_suit.len() == 1 && same_suit[0] < ace;
            prop_assert_eq!(Lead::parse(lead, Signal::Altra, &hand).is_ok(), expected);
        }
    }
}
