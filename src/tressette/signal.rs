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
    /// The leader holds two or more other cards of the suit, all King or
    /// lower.
    Striscio,
    /// Asks the leader's partner to play its best card of the suit.
    Busso,
    /// The leader keeps the Two of the suit and holds neither its Three nor
    /// its Ace.
    Ribusso,
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
        let ace = TressetteCard::new(ItalianRank::Ace, card.suit());
        let (others, all_low) = hand
            .iter()
            .filter(|held| held.suit() == card.suit() && **held != card)
            .fold((0, true), |(count, all_low), held| {
                (count + 1, all_low && *held < ace)
            });
        let matches = match signal {
            Signal::Volo => others == 0,
            Signal::Altra => others == 1 && all_low,
            Signal::Striscio => others >= 2 && all_low,
            Signal::Busso => true,
            Signal::Ribusso => {
                let two = TressetteCard::new(ItalianRank::Two, card.suit());
                let three = TressetteCard::new(ItalianRank::Three, card.suit());
                card != two && hand.contains(&two) && !hand.contains(&three) && !hand.contains(&ace)
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

        /// Striscio is true exactly when the leader holds two or more other
        /// cards of the suit led, all King or lower.
        #[test]
        fn striscio_is_true_exactly_when_only_low_cards_of_the_suit_are_left(
            hand in hand_strategy(),
            pick in any::<Index>(),
        ) {
            let lead = *pick.get(&hand);
            let ace = TressetteCard::new(ItalianRank::Ace, lead.suit());
            let same_suit = same_suit(&hand, lead);
            let expected = same_suit.len() >= 2 && same_suit.iter().all(|card| *card < ace);
            prop_assert_eq!(Lead::parse(lead, Signal::Striscio, &hand).is_ok(), expected);
        }

        /// Busso asks for the partner's best card, so any lead may give it.
        #[test]
        fn busso_is_always_true(hand in hand_strategy(), pick in any::<Index>()) {
            let lead = *pick.get(&hand);
            prop_assert!(Lead::parse(lead, Signal::Busso, &hand).is_ok());
        }

        /// Ribusso is true exactly when the leader keeps the Two of the suit led
        /// and holds neither its Three nor its Ace, the lead included.
        #[test]
        fn ribusso_is_true_exactly_when_the_two_is_kept_without_the_three_or_the_ace(
            hand in hand_strategy(),
            pick in any::<Index>(),
        ) {
            let lead = *pick.get(&hand);
            let two = TressetteCard::new(ItalianRank::Two, lead.suit());
            let three = TressetteCard::new(ItalianRank::Three, lead.suit());
            let ace = TressetteCard::new(ItalianRank::Ace, lead.suit());
            let expected = lead != two
                && hand.contains(&two)
                && !hand.contains(&three)
                && !hand.contains(&ace);
            prop_assert_eq!(Lead::parse(lead, Signal::Ribusso, &hand).is_ok(), expected);
        }
    }
}
