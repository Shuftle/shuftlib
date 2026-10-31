//! Signals of Tressette: what the seat leading a trick may say about the suit
//! it leads, such as "volo" for its last card of that suit.
//!
//! Signals are never bluffs: a signal is only accepted when the leader's hand
//! matches it.

use std::{
    fmt::{self, Display},
    ops::{Index, IndexMut},
};

use crate::cards::italian::{ItalianRank, PerSuit};

use super::{
    Error, TressetteCard, Trick,
    trick::{Held, Opening, Play},
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
    /// its Ace. Only the first lead of a suit in a hand may carry it.
    Ribusso,
}

impl Display for Signal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Volo => "volo",
            Self::Altra => "altra",
            Self::Striscio => "striscio",
            Self::Busso => "busso",
            Self::Ribusso => "ribusso",
        };
        f.write_str(name)
    }
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

/// A value for each signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct PerSignal<T> {
    volo: T,
    altra: T,
    striscio: T,
    busso: T,
    ribusso: T,
}

impl<T> Index<Signal> for PerSignal<T> {
    type Output = T;

    fn index(&self, signal: Signal) -> &T {
        match signal {
            Signal::Volo => &self.volo,
            Signal::Altra => &self.altra,
            Signal::Striscio => &self.striscio,
            Signal::Busso => &self.busso,
            Signal::Ribusso => &self.ribusso,
        }
    }
}

impl<T> IndexMut<Signal> for PerSignal<T> {
    fn index_mut(&mut self, signal: Signal) -> &mut T {
        match signal {
            Signal::Volo => &mut self.volo,
            Signal::Altra => &mut self.altra,
            Signal::Striscio => &mut self.striscio,
            Signal::Busso => &mut self.busso,
            Signal::Ribusso => &mut self.ribusso,
        }
    }
}

/// The signals a variant allows. The default allows none: the game is played
/// without signals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Signals(PerSignal<bool>);

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
            allowed.0[signal] = true;
        }
        allowed
    }

    /// Returns whether `signal` is allowed.
    pub fn allows(&self, signal: Signal) -> bool {
        self.0[signal]
    }

    /// Returns whether no signal is allowed: the game is played without them.
    pub fn is_muto(self) -> bool {
        self == Self::default()
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

/// What a leader's signal is read against, besides its hand.
pub(super) struct Context {
    /// The signals the variant allows.
    pub rules: Signals,
    /// The suits the leader already signalled in this hand.
    pub signalled: PerSuit<bool>,
    /// The suits that opened a trick so far this hand.
    pub led: PerSuit<bool>,
}

/// A signal the variant allows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct AllowedSignal(Signal);

impl AllowedSignal {
    /// Builds `signal` as allowed by `rules`, or returns why it isn't.
    pub(crate) fn new(signal: Signal, rules: Signals) -> Result<Self, Error> {
        if rules.allows(signal) {
            return Ok(Self(signal));
        }
        let reason = if rules.is_muto() {
            SignalRefusal::Muto
        } else {
            SignalRefusal::NotAllowed
        };
        Err(Error::CannotSignal { signal, reason })
    }
}

impl<'a> Opening<'a> {
    /// Gives `allowed` with the lead, or returns that the leader already
    /// signalled in its suit, as marked in `signalled`.
    pub(crate) fn signal(
        self,
        allowed: AllowedSignal,
        signalled: PerSuit<bool>,
    ) -> Result<Unrepeated<'a>, Error> {
        let signal = allowed.0;
        if signalled[self.card().suit()] {
            return Err(Error::CannotSignal {
                signal,
                reason: SignalRefusal::AlreadySignalled,
            });
        }
        Ok(Unrepeated {
            opening: self,
            signal,
        })
    }
}

/// An allowed signal on a lead, the first in its suit from this leader.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Unrepeated<'a> {
    opening: Opening<'a>,
    signal: Signal,
}

impl<'a> Unrepeated<'a> {
    /// Checks the signal against the suits marked in `led`, or returns that
    /// a ribusso comes after the suit's first lead.
    pub(crate) fn timely(self, led: PerSuit<bool>) -> Result<Timely<'a>, Error> {
        if self.signal == Signal::Ribusso && led[self.opening.card().suit()] {
            return Err(Error::CannotSignal {
                signal: self.signal,
                reason: SignalRefusal::SuitAlreadyLed,
            });
        }
        Ok(Timely(self))
    }
}

/// An unrepeated signal on a lead, given when the rules on earlier leads
/// allow it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Timely<'a>(Unrepeated<'a>);

impl Timely<'_> {
    /// Builds the lead, or returns that the leader's hand doesn't match the
    /// signal.
    pub(crate) fn lead(self) -> Result<Lead, Error> {
        let Unrepeated { opening, signal } = self.0;
        let card = opening.card();
        let hand = opening.hand();
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
            Ok(Lead {
                play: opening.into(),
                signal,
            })
        } else {
            Err(Error::CannotSignal {
                signal,
                reason: SignalRefusal::HandDoesntMatch,
            })
        }
    }
}

/// The card that opens a trick, with a signal the leader's hand matches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Lead {
    play: Play,
    signal: Signal,
}

impl Lead {
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

/// Builds the lead of `opening` with `signal` in `context`, through each
/// rule a signal must meet.
pub(super) fn lead(opening: Opening<'_>, signal: Signal, context: &Context) -> Result<Lead, Error> {
    opening
        .signal(
            AllowedSignal::new(signal, context.rules)?,
            context.signalled,
        )?
        .timely(context.led)?
        .lead()
}

/// Returns every lead with a signal the seat holding `hand` may make on
/// `trick` in `context`.
pub(super) fn allowed(context: &Context, hand: &[TressetteCard], trick: &Trick) -> Vec<Lead> {
    hand.iter()
        .filter_map(|&card| Held::new(card, hand).and_then(|held| held.open(trick)).ok())
        .flat_map(|opening| Signal::ALL.map(|signal| (opening, signal)))
        .filter_map(|(opening, signal)| lead(opening, signal, context).ok())
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

    /// Returns `lead`, a card of `hand`, as the opening of an empty trick.
    fn opening(lead: TressetteCard, hand: &[TressetteCard]) -> Opening<'_> {
        Held::new(lead, hand)
            .and_then(|held| held.open(&Trick::new(Seat::North)))
            .expect("the lead is in the hand, and the trick is empty")
    }

    /// Returns whether the leader holding `hand` may lead `lead` with
    /// `signal` when every signal is allowed and none was given before.
    fn matches(lead: TressetteCard, hand: &[TressetteCard], signal: Signal) -> bool {
        let fresh = Context {
            rules: Signals::all(),
            signalled: PerSuit::default(),
            led: PerSuit::default(),
        };
        super::lead(opening(lead, hand), signal, &fresh).is_ok()
    }

    /// Without signals every signal is refused as muto, and with all of them
    /// every signal is allowed.
    #[test]
    fn none_refuses_every_signal_and_all_allows_every_one() {
        for signal in Signal::ALL {
            assert_eq!(
                AllowedSignal::new(signal, Signals::none()).map(|_| ()),
                Err(Error::CannotSignal {
                    signal,
                    reason: SignalRefusal::Muto
                })
            );
            assert!(AllowedSignal::new(signal, Signals::all()).is_ok());
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
            prop_assert_eq!(matches(lead, &hand, Signal::Volo), expected);
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
            prop_assert_eq!(matches(lead, &hand, Signal::Altra), expected);
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
            prop_assert_eq!(matches(lead, &hand, Signal::Striscio), expected);
        }

        /// Busso asks for the partner's best card, so any lead may give it.
        #[test]
        fn busso_is_always_true(hand in hand_strategy(), pick in any::<Index>()) {
            let lead = *pick.get(&hand);
            prop_assert!(matches(lead, &hand, Signal::Busso));
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
            prop_assert_eq!(matches(lead, &hand, Signal::Ribusso), expected);
        }

        /// A signal is allowed exactly when the rules list it; refused, it
        /// tells a game without signals from a signal left out.
        #[test]
        fn a_signal_is_allowed_exactly_when_the_rules_list_it(
            signal in signal_strategy(),
            allowed in signals_strategy(),
        ) {
            let refused = |reason| Err(Error::CannotSignal { signal, reason });
            let expected = if allowed.contains(&signal) {
                Ok(())
            } else if allowed.is_empty() {
                refused(SignalRefusal::Muto)
            } else {
                refused(SignalRefusal::NotAllowed)
            };
            let rules = Signals::only(allowed);
            prop_assert_eq!(AllowedSignal::new(signal, rules).map(|_| ()), expected);
        }

        /// An allowed signal can be given on a lead exactly when the leader
        /// hasn't signalled in its suit yet; other suits change nothing.
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
                Ok(())
            };
            let allowed = AllowedSignal::new(signal, Signals::all()).expect("every signal is allowed");
            let given = opening(lead, &hand).signal(allowed, signalled);
            prop_assert_eq!(given.map(|_| ()), expected);
        }

        /// Once a suit has been led, a ribusso can't be given in it; every
        /// other signal, and ribusso in a suit not led yet, is timely.
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
                Ok(())
            };
            let allowed = AllowedSignal::new(signal, Signals::all()).expect("every signal is allowed");
            let unrepeated = opening(lead, &hand)
                .signal(allowed, PerSuit::default())
                .expect("nothing was signalled yet");
            prop_assert_eq!(unrepeated.timely(led).map(|_| ()), expected);
        }
    }
}
