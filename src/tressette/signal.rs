//! Signals of Tressette: what the seat leading a trick may say about the suit
//! it leads, such as "volo" for its last card of that suit.
//!
//! Signals are never bluffs: a signal is only accepted when the leader's hand
//! matches it.

use crate::cards::italian::{ItalianRank, PerSuit};

use super::{
    Error, TressetteCard, Trick,
    trick::{Opening, Play},
};

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
pub struct Signals {
    volo: bool,
    altra: bool,
    striscio: bool,
    busso: bool,
    ribusso: bool,
}

impl Signals {
    /// Allows every signal.
    pub fn all() -> Self {
        Self::only(Signal::ALL)
    }

    /// Allows no signal: the game is played without them.
    pub fn none() -> Self {
        Self::default()
    }

    /// Allows exactly `signals`.
    ///
    /// ```
    /// use shuftlib::tressette::signal::{Signal, Signals};
    ///
    /// let signals = Signals::only([Signal::Busso, Signal::Volo]);
    /// assert!(signals.allows(Signal::Busso) && !signals.allows(Signal::Striscio));
    /// ```
    pub fn only(signals: impl IntoIterator<Item = Signal>) -> Self {
        let mut allowed = Self::default();
        for signal in signals {
            *allowed.flag(signal) = true;
        }
        allowed
    }

    /// Returns whether `signal` is allowed.
    pub fn allows(&self, signal: Signal) -> bool {
        match signal {
            Signal::Volo => self.volo,
            Signal::Altra => self.altra,
            Signal::Striscio => self.striscio,
            Signal::Busso => self.busso,
            Signal::Ribusso => self.ribusso,
        }
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
    pub rules: Signals,
    /// The leader's cards before it leads.
    pub hand: &'a [TressetteCard],
    /// The suits the leader already signalled in this hand.
    pub signalled: PerSuit<bool>,
    /// The suits that opened a trick so far this hand.
    pub led: PerSuit<bool>,
}

/// The card that opens a trick, with a signal the leader's hand matches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Lead {
    opening: Opening,
    signal: Signal,
}

impl Lead {
    /// Builds the lead of `opening` with `signal` in `context`, or returns
    /// why the signal can't be given.
    pub(super) fn new(opening: Opening, signal: Signal, context: &Context) -> Result<Self, Error> {
        let card = opening.card();
        if !context.rules.allows(signal) {
            let reason = if context.rules.is_muto() {
                SignalRefusal::Muto
            } else {
                SignalRefusal::NotAllowed
            };
            return Err(Error::CannotSignal { signal, reason });
        }
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
            Ok(Self { opening, signal })
        } else {
            Err(Error::CannotSignal {
                signal,
                reason: SignalRefusal::HandDoesntMatch,
            })
        }
    }

    /// Returns the card led.
    pub(crate) fn card(&self) -> TressetteCard {
        self.opening.card()
    }

    /// Returns the card led, as a legal play.
    pub(crate) fn play(&self) -> Play {
        self.opening.into()
    }

    /// Returns the signal given.
    pub(crate) fn signal(&self) -> Signal {
        self.signal
    }
}

/// Returns every lead with a signal the leader may make on `trick` in
/// `context`.
pub(super) fn allowed(context: &Context, trick: &Trick) -> Vec<Lead> {
    context
        .hand
        .iter()
        .filter_map(|&card| Opening::new(card, context.hand, trick).ok())
        .flat_map(|opening| Signal::ALL.map(|signal| (opening, signal)))
        .filter_map(|(opening, signal)| Lead::new(opening, signal, context).ok())
        .collect()
}

#[cfg(test)]
/// Test utilities for the signal module.
pub mod test_utils {
    use super::{Signal, Signals};
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
    pub fn rules_strategy() -> impl Strategy<Value = Signals> {
        signals_strategy().prop_map(Signals::only)
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
    use crate::tressette::{Seat, test_utils::hand_strategy};
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
            rules: Signals::all(),
            hand,
            signalled: PerSuit::default(),
            led: PerSuit::default(),
        }
    }

    /// Returns `lead`, a card of `hand`, as the opening of an empty trick.
    fn opening(lead: TressetteCard, hand: &[TressetteCard]) -> Opening {
        Opening::new(lead, hand, &Trick::new(Seat::North)).expect("the lead is in the hand")
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
            prop_assert_eq!(Lead::new(opening(lead, &hand), Signal::Volo, &any_signal(&hand)).is_ok(), expected);
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
            prop_assert_eq!(Lead::new(opening(lead, &hand), Signal::Altra, &any_signal(&hand)).is_ok(), expected);
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
            prop_assert_eq!(Lead::new(opening(lead, &hand), Signal::Striscio, &any_signal(&hand)).is_ok(), expected);
        }

        /// Busso asks for the partner's best card, so any lead may give it.
        #[test]
        fn busso_is_always_true(hand in hand_strategy(), pick in any::<Index>()) {
            let lead = *pick.get(&hand);
            prop_assert!(Lead::new(opening(lead, &hand), Signal::Busso, &any_signal(&hand)).is_ok());
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
            prop_assert_eq!(Lead::new(opening(lead, &hand), Signal::Ribusso, &any_signal(&hand)).is_ok(), expected);
        }

        /// `only` allows exactly the signals it's given, `all` every one, and
        /// `none` none.
        #[test]
        fn only_allows_exactly_the_signals_given(allowed in signals_strategy()) {
            let only = Signals::only(allowed.iter().copied());
            for signal in Signal::ALL {
                prop_assert_eq!(only.allows(signal), allowed.contains(&signal));
                prop_assert!(Signals::all().allows(signal));
                prop_assert!(!Signals::none().allows(signal));
            }
        }

        /// A signal can be given exactly when the rules allow it and the hand
        /// matches it, and a refusal says which of the two failed, the rules
        /// first.
        #[test]
        fn a_signal_is_given_exactly_when_allowed_and_true(
            hand in hand_strategy(),
            pick in any::<Index>(),
            signal in signal_strategy(),
            allowed in signals_strategy(),
        ) {
            let lead = *pick.get(&hand);
            let rules = Signals::only(allowed.iter().copied());
            let matches_hand = Lead::new(opening(lead, &hand), signal, &any_signal(&hand)).is_ok();
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
                signalled: PerSuit::default(),
                led: PerSuit::default(),
            };
            prop_assert_eq!(Lead::new(opening(lead, &hand), signal, &context).map(|_| ()), expected);
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
                Lead::new(opening(lead, &hand), signal, &any_signal(&hand)).map(|_| ())
            };
            let context = Context { signalled, ..any_signal(&hand) };
            prop_assert_eq!(Lead::new(opening(lead, &hand), signal, &context).map(|_| ()), expected);
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
                Lead::new(opening(lead, &hand), signal, &any_signal(&hand)).map(|_| ())
            };
            let context = Context { led, ..any_signal(&hand) };
            prop_assert_eq!(Lead::new(opening(lead, &hand), signal, &context).map(|_| ()), expected);
        }
    }
}
