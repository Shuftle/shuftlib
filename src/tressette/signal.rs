//! Signals of Tressette: what the seat leading a trick may say about the suit
//! it leads, such as "volo" for its last card of that suit.
//!
//! Signals are never bluffs: a signal is only accepted when the leader's hand
//! matches it.

use crate::cards::italian::{ItalianRank, PerSuit};

use super::{Error, TressetteCard, Trick, trick::Play};

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

impl Signal {
    /// Every signal there is.
    pub const ALL: [Self; 5] = [
        Self::Volo,
        Self::Altra,
        Self::Striscio,
        Self::Busso,
        Self::Ribusso,
    ];
}

/// The signals a variant allows. The default allows none: the game is played
/// without signals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Rules {
    volo: bool,
    altra: bool,
    striscio: bool,
    busso: bool,
    ribusso: bool,
}

impl Rules {
    /// Returns these rules with `signal` allowed as well.
    pub fn allow(mut self, signal: Signal) -> Self {
        *self.flag(signal) = true;
        self
    }

    /// Returns whether `signal` is allowed.
    pub fn allows(mut self, signal: Signal) -> bool {
        *self.flag(signal)
    }

    /// Returns whether no signal is allowed: the game is played without them.
    pub fn is_muto(self) -> bool {
        self == Self::default()
    }

    /// Returns the flag that says whether `signal` is allowed.
    fn flag(&mut self, signal: Signal) -> &mut bool {
        match signal {
            Signal::Volo => &mut self.volo,
            Signal::Altra => &mut self.altra,
            Signal::Striscio => &mut self.striscio,
            Signal::Busso => &mut self.busso,
            Signal::Ribusso => &mut self.ribusso,
        }
    }
}

/// Why a signal can't be given.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
pub enum SignalRefusal {
    /// The variant allows no signals: the game is played without them.
    #[error("the game is played without signals")]
    Muto,
    /// The variant allows other signals, but not this one.
    #[error("this variant doesn't allow it")]
    NotAllowed,
    /// The leader's hand doesn't match it.
    #[error("the hand doesn't match it")]
    HandDoesntMatch,
    /// The seat isn't leading the trick: only the leader may signal.
    #[error("only the seat leading the trick may signal")]
    NotLeading,
    /// The leader already signalled in this suit this hand.
    #[error("a signal was already given in this suit")]
    AlreadySignalled,
    /// The suit was already led this hand, and only the first lead of a suit
    /// may carry a ribusso.
    #[error("ribusso is only for the first lead of a suit")]
    SuitAlreadyLed,
}

/// What a leader's signal is read against.
pub(super) struct Context<'a> {
    /// The signals the variant allows.
    pub rules: Rules,
    /// The leader's cards before it leads.
    pub hand: &'a [TressetteCard],
    /// The trick the signal would be given on.
    pub trick: Trick,
    /// The suits the leader already signalled in this hand.
    pub signalled: PerSuit<bool>,
    /// The suits that opened a trick so far this hand.
    pub led: PerSuit<bool>,
}

/// The card that opens a trick, with the signal given alongside it. It can
/// only be built from a hand that matches the signal, so every `Lead` is
/// honest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Lead {
    play: Play,
    signal: Signal,
}

impl Lead {
    /// Reads `signal` on `card` against `context`, or says why it can't be
    /// given.
    pub(super) fn new(
        card: TressetteCard,
        signal: Signal,
        context: &Context,
    ) -> Result<Self, Error> {
        if context.trick.led_suit().is_some() {
            return Err(Error::CannotSignal {
                signal,
                reason: SignalRefusal::NotLeading,
            });
        }
        if !context.rules.allows(signal) {
            let reason = if context.rules.is_muto() {
                SignalRefusal::Muto
            } else {
                SignalRefusal::NotAllowed
            };
            return Err(Error::CannotSignal { signal, reason });
        }
        let play = Play::new(card, context.hand, &context.trick)?;
        if context.signalled[card.suit()] {
            return Err(Error::CannotSignal {
                signal,
                reason: SignalRefusal::AlreadySignalled,
            });
        }
        if signal == Signal::Ribusso && context.led[card.suit()] {
            return Err(Error::CannotSignal {
                signal,
                reason: SignalRefusal::SuitAlreadyLed,
            });
        }
        let hand = context.hand;
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
            Ok(Self { play, signal })
        } else {
            Err(Error::CannotSignal {
                signal,
                reason: SignalRefusal::HandDoesntMatch,
            })
        }
    }

    /// Returns the card led.
    pub(crate) fn card(&self) -> TressetteCard {
        self.play.card()
    }

    /// Returns the card led, as a legal play.
    pub(crate) fn play(&self) -> Play {
        self.play
    }

    /// Returns the signal given.
    pub(crate) fn signal(&self) -> Signal {
        self.signal
    }
}

/// Returns every lead with a signal the leader may make in `context`.
pub(super) fn allowed(context: &Context) -> Vec<Lead> {
    context
        .hand
        .iter()
        .flat_map(|&card| Signal::ALL.map(|signal| (card, signal)))
        .filter_map(|(card, signal)| Lead::new(card, signal, context).ok())
        .collect()
}

#[cfg(test)]
/// Test utilities for the signal module.
pub mod test_utils {
    use super::{Rules, Signal};
    use proptest::{prelude::*, sample};

    /// Generates any signal.
    pub fn signal_strategy() -> impl Strategy<Value = Signal> {
        sample::select(Signal::ALL.to_vec())
    }

    /// Generates any set of distinct signals.
    pub fn signals_strategy() -> impl Strategy<Value = Vec<Signal>> {
        sample::subsequence(Signal::ALL.to_vec(), 0..=Signal::ALL.len())
    }

    /// Generates any signal rules, from none allowed to all of them.
    pub fn rules_strategy() -> impl Strategy<Value = Rules> {
        signals_strategy()
            .prop_map(|allowed| allowed.into_iter().fold(Rules::default(), Rules::allow))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        test_utils::{signal_strategy, signals_strategy},
        *,
    };
    use crate::cards::italian::ItalianRank;
    use crate::cards::italian::test_utils::per_suit_strategy;
    use crate::tressette::{
        Seat, card::test_utils::tressette_card_strategy, table::test_utils::seat_strategy,
        test_utils::hand_strategy,
    };
    use proptest::{prelude::*, sample::Index};

    /// Returns the cards of `hand` in `lead`'s suit, other than `lead`.
    fn same_suit(hand: &[TressetteCard], lead: TressetteCard) -> Vec<TressetteCard> {
        hand.iter()
            .copied()
            .filter(|card| card.suit() == lead.suit() && *card != lead)
            .collect()
    }

    /// A context for `hand` in which every signal is allowed.
    fn any_signal(hand: &[TressetteCard]) -> Context<'_> {
        Context {
            rules: Signal::ALL.into_iter().fold(Rules::default(), Rules::allow),
            hand,
            trick: Trick::new(Seat::North),
            signalled: PerSuit::default(),
            led: PerSuit::default(),
        }
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
            prop_assert_eq!(Lead::new(lead, Signal::Volo, &any_signal(&hand)).is_ok(), expected);
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
            prop_assert_eq!(Lead::new(lead, Signal::Altra, &any_signal(&hand)).is_ok(), expected);
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
            prop_assert_eq!(Lead::new(lead, Signal::Striscio, &any_signal(&hand)).is_ok(), expected);
        }

        /// Busso asks for the partner's best card, so any lead may give it.
        #[test]
        fn busso_is_always_true(hand in hand_strategy(), pick in any::<Index>()) {
            let lead = *pick.get(&hand);
            prop_assert!(Lead::new(lead, Signal::Busso, &any_signal(&hand)).is_ok());
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
            prop_assert_eq!(Lead::new(lead, Signal::Ribusso, &any_signal(&hand)).is_ok(), expected);
        }

        /// A signal can be given exactly when the rules allow it and the hand
        /// matches it, and a refusal says which of the two failed, the rules
        /// first, telling a game without signals from a signal left out.
        #[test]
        fn a_signal_is_given_exactly_when_allowed_and_true(
            hand in hand_strategy(),
            pick in any::<Index>(),
            signal in signal_strategy(),
            allowed in signals_strategy(),
        ) {
            let lead = *pick.get(&hand);
            let rules = allowed.iter().copied().fold(Rules::default(), Rules::allow);
            let matches_hand = Lead::new(lead, signal, &any_signal(&hand)).is_ok();
            let refused = |reason| Err(Error::CannotSignal { signal, reason });
            let expected = if allowed.is_empty() {
                refused(SignalRefusal::Muto)
            } else if !allowed.contains(&signal) {
                refused(SignalRefusal::NotAllowed)
            } else if matches_hand {
                Ok(())
            } else {
                refused(SignalRefusal::HandDoesntMatch)
            };
            let context = Context {
                rules,
                hand: &hand,
                trick: Trick::new(Seat::North),
                signalled: PerSuit::default(),
                led: PerSuit::default(),
            };
            prop_assert_eq!(Lead::new(lead, signal, &context).map(|_| ()), expected);
        }

        /// A leader that already signalled in the suit led can't signal in it
        /// again; signals in other suits change nothing.
        #[test]
        fn a_seat_signals_once_per_suit(
            hand in hand_strategy(),
            pick in any::<Index>(),
            signal in signal_strategy(),
            signalled in per_suit_strategy(any::<bool>()),
        ) {
            let lead = *pick.get(&hand);
            let expected = if signalled[lead.suit()] {
                Err(Error::CannotSignal { signal, reason: SignalRefusal::AlreadySignalled })
            } else {
                Lead::new(lead, signal, &any_signal(&hand)).map(|_| ())
            };
            let context = Context { signalled, ..any_signal(&hand) };
            prop_assert_eq!(Lead::new(lead, signal, &context).map(|_| ()), expected);
        }

        /// Once a suit has been led, a ribusso can't be given in it; every
        /// other signal, and ribusso in a suit not led yet, is unaffected.
        #[test]
        fn ribusso_is_only_for_the_first_lead_of_a_suit(
            hand in hand_strategy(),
            pick in any::<Index>(),
            signal in signal_strategy(),
            led in per_suit_strategy(any::<bool>()),
        ) {
            let lead = *pick.get(&hand);
            let expected = if signal == Signal::Ribusso && led[lead.suit()] {
                Err(Error::CannotSignal { signal, reason: SignalRefusal::SuitAlreadyLed })
            } else {
                Lead::new(lead, signal, &any_signal(&hand)).map(|_| ())
            };
            let context = Context { led, ..any_signal(&hand) };
            prop_assert_eq!(Lead::new(lead, signal, &context).map(|_| ()), expected);
        }

        /// Once a card is on the trick, no signal can be given, whatever the
        /// rules and the hand.
        #[test]
        fn only_the_leader_signals(
            hand in hand_strategy(),
            pick in any::<Index>(),
            signal in signal_strategy(),
            allowed in signals_strategy(),
            leader in seat_strategy(),
            played in tressette_card_strategy(),
        ) {
            let lead = *pick.get(&hand);
            let mut trick = Trick::new(leader);
            trick.play(played);
            let context = Context {
                rules: allowed.into_iter().fold(Rules::default(), Rules::allow),
                hand: &hand,
                trick,
                signalled: PerSuit::default(),
                led: PerSuit::default(),
            };
            prop_assert_eq!(
                Lead::new(lead, signal, &context).map(|_| ()),
                Err(Error::CannotSignal { signal, reason: SignalRefusal::NotLeading })
            );
        }
    }
}
