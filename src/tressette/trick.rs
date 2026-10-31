//! Tricks: the one being played, and completed ones.

use crate::cards::italian::Suit;

use super::{
    Error, Seat, TressetteCard,
    signal::{self, Signal},
};

/// The trick being played: who led it, and the cards played so far.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Trick {
    leader: Seat,
    /// In play order, starting from the leader, then `None` for the seats
    /// yet to play. There's no room for the last card: a trick is replaced
    /// by the next one as soon as it's played.
    cards: [Option<TressetteCard>; 3],
    /// The signal given with the lead, if any.
    signal: Option<Signal>,
}

/// A trick whose four cards have been played.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CompletedTrick {
    leader: Seat,
    /// In play order, starting from the leader.
    cards: [TressetteCard; 4],
    taker: Seat,
    /// The signal given with the lead, if any.
    signal: Option<Signal>,
}

impl Trick {
    pub(super) fn new(leader: Seat) -> Self {
        Self {
            leader,
            cards: [None; 3],
            signal: None,
        }
    }

    /// Plays `held` for [`next_to_play`](Self::next_to_play), or returns that
    /// it must follow the led suit. If that completes the trick, returns it
    /// and starts the next one, led by the taker.
    pub(super) fn follow(&mut self, held: Held<'_>) -> Result<Option<CompletedTrick>, Error> {
        let play = held.follow(self)?;
        Ok(self.play(play))
    }

    /// Leads the trick with `held`, giving `signal`, or returns why the lead
    /// or the signal isn't allowed in `context`.
    pub(super) fn lead(
        &mut self,
        held: Held<'_>,
        signal: Signal,
        context: &signal::Context,
    ) -> Result<(), Error> {
        let lead = signal::lead(held.open(self)?, signal, context)?;
        self.cards[0] = Some(lead.card());
        self.signal = Some(lead.signal());
        Ok(())
    }

    /// Returns the seat that led, or will lead, the trick.
    pub fn leader(&self) -> Seat {
        self.leader
    }

    /// Returns each card played so far with the seat that played it, in play
    /// order.
    pub fn plays(&self) -> impl Iterator<Item = (Seat, TressetteCard)> + '_ {
        plays(self.leader, self.played())
    }

    /// Returns the suit of the card that led the trick, if any was played.
    pub fn led_suit(&self) -> Option<Suit> {
        self.cards[0].map(|card| card.suit())
    }

    /// Returns the signal given with the lead, if any.
    pub fn signal(&self) -> Option<Signal> {
        self.signal
    }

    /// Returns the seat whose turn it is.
    pub fn next_to_play(&self) -> Seat {
        self.leader.after(self.played().count())
    }

    /// Returns the cards played so far, in play order.
    fn played(&self) -> impl Iterator<Item = TressetteCard> + '_ {
        self.cards.iter().map_while(|&card| card)
    }

    /// Plays `play` for [`next_to_play`](Self::next_to_play). If that
    /// completes the trick, returns it and starts the next one, led by the
    /// taker.
    fn play(&mut self, play: Play) -> Option<CompletedTrick> {
        let card = play.card();
        match self.cards {
            [None, _, _] => self.cards[0] = Some(card),
            [Some(_), None, _] => self.cards[1] = Some(card),
            [Some(_), Some(_), None] => self.cards[2] = Some(card),
            [Some(first), Some(second), Some(third)] => {
                return Some(self.complete([first, second, third, card]));
            }
        }
        None
    }

    /// Completes the trick with its four `cards`, in play order, and starts
    /// the next one, led by the taker.
    fn complete(&mut self, cards: [TressetteCard; 4]) -> CompletedTrick {
        let [first, ..] = cards;
        let leader = self.leader;
        // The lead follows its own suit: it's the strongest card until a
        // stronger one of its suit comes.
        let (taker, _) = plays(leader, cards).fold((leader, first), |best, (seat, card)| {
            if card.suit() == first.suit() && card > best.1 {
                (seat, card)
            } else {
                best
            }
        });
        let signal = self.signal;
        *self = Self::new(taker);
        CompletedTrick {
            leader,
            cards,
            taker,
            signal,
        }
    }
}

impl CompletedTrick {
    /// Returns the seat that led the trick.
    pub fn leader(&self) -> Seat {
        self.leader
    }

    /// Returns the seat that took the trick: the one that played the
    /// strongest card of the led suit.
    pub fn taker(&self) -> Seat {
        self.taker
    }

    /// Returns the signal given with the lead, if any.
    pub fn signal(&self) -> Option<Signal> {
        self.signal
    }

    /// Returns each card with the seat that played it, in play order.
    pub fn plays(&self) -> impl Iterator<Item = (Seat, TressetteCard)> + use<> {
        plays(self.leader, self.cards)
    }

    /// Returns the cards of the trick, in play order.
    pub fn cards(&self) -> [TressetteCard; 4] {
        self.cards
    }
}

/// Pairs `cards`, played in order from `leader`, with the seats that played
/// them.
fn plays<I: IntoIterator<Item = TressetteCard>>(
    leader: Seat,
    cards: I,
) -> impl Iterator<Item = (Seat, TressetteCard)> + use<I> {
    cards
        .into_iter()
        .enumerate()
        .map(move |(turn, card)| (leader.after(turn), card))
}

/// A card a seat holds, with the hand it's in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Held<'a> {
    card: TressetteCard,
    hand: &'a [TressetteCard],
}

impl<'a> Held<'a> {
    /// Builds `card` as held in `hand`, or returns that it isn't.
    pub(crate) fn new(card: TressetteCard, hand: &'a [TressetteCard]) -> Result<Self, Error> {
        if hand.contains(&card) {
            Ok(Self { card, hand })
        } else {
            Err(Error::CardNotInHand { card })
        }
    }

    /// Plays the card on `trick`, or returns that it must follow the led
    /// suit.
    pub(crate) fn follow(self, trick: &Trick) -> Result<Play, Error> {
        if let Some(required) = trick.led_suit()
            && self.card.suit() != required
            && self.hand.iter().any(|held| held.suit() == required)
        {
            return Err(Error::MustFollowSuit {
                card: self.card,
                required,
            });
        }
        Ok(Play(self.card))
    }

    /// Leads the card on `trick`, or returns that the trick has already been
    /// led.
    pub(crate) fn open(self, trick: &Trick) -> Result<Opening<'a>, Error> {
        if trick.led_suit().is_some() {
            Err(Error::NotLeading)
        } else {
            Ok(Opening(self))
        }
    }
}

/// A card a seat may play on the trick: it holds it, and it follows the led
/// suit if it can.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Play(TressetteCard);

impl Play {
    /// Returns the card played.
    pub(crate) fn card(self) -> TressetteCard {
        self.0
    }
}

/// A card a seat leads: it holds it, and the trick has no card yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Opening<'a>(Held<'a>);

impl<'a> Opening<'a> {
    /// Returns the card led.
    pub(crate) fn card(self) -> TressetteCard {
        self.0.card
    }

    /// Returns the leader's hand, the card led included.
    pub(crate) fn hand(self) -> &'a [TressetteCard] {
        self.0.hand
    }
}

impl From<Opening<'_>> for Play {
    fn from(opening: Opening<'_>) -> Self {
        Self(opening.card())
    }
}

#[cfg(test)]
/// Test utilities for the trick module.
pub mod test_utils {
    use super::{Play, TressetteCard, Trick};
    use crate::tressette::{deck, table::test_utils::seat_strategy};
    use proptest::{prelude::*, sample};

    /// Returns `card` as played, whether or not it could be.
    pub(crate) fn played(card: TressetteCard) -> Play {
        Play(card)
    }

    /// Generates a trick being played, from any leader, with up to three
    /// cards on it.
    pub fn trick_strategy() -> impl Strategy<Value = Trick> {
        let cards = sample::subsequence(deck().into_iter().collect::<Vec<_>>(), 0..=3);
        (seat_strategy(), cards).prop_map(|(leader, cards)| {
            let mut trick = Trick::new(leader);
            for card in cards {
                trick.play(played(card));
            }
            trick
        })
    }
}

#[cfg(test)]
mod tests {
    use proptest::{prelude::*, sample};

    use super::{
        test_utils::{played, trick_strategy},
        *,
    };
    use crate::{
        cards::italian::ItalianRank,
        tressette::{deck, table::test_utils::seat_strategy, test_utils::hand_strategy},
    };

    fn card(rank: ItalianRank, suit: Suit) -> TressetteCard {
        TressetteCard::new(rank, suit)
    }

    #[test]
    fn plays_are_attributed_from_the_leader() {
        let mut trick = Trick::new(Seat::West);
        let ace = card(ItalianRank::Ace, Suit::Clubs);
        let two = card(ItalianRank::Two, Suit::Clubs);
        assert_eq!(trick.play(played(ace)), None);
        assert_eq!(trick.play(played(two)), None);
        assert_eq!(
            trick.plays().collect::<Vec<_>>(),
            [(Seat::West, ace), (Seat::South, two)]
        );
        assert_eq!(trick.led_suit(), Some(Suit::Clubs));
    }

    proptest! {
        /// The strongest card of the led suit takes the trick, whoever leads
        /// and whatever is played, and the taker leads the next one.
        #[test]
        fn the_strongest_card_of_the_led_suit_takes(
            leader in seat_strategy(),
            cards in sample::subsequence(deck().into_iter().collect::<Vec<_>>(), 4).prop_shuffle(),
        ) {
            let mut trick = Trick::new(leader);
            let completed = cards.iter().find_map(|&card| trick.play(played(card))).unwrap();
            let led = cards[0].suit();
            let (_, taken_with) = completed
                .plays()
                .find(|&(seat, _)| seat == completed.taker())
                .unwrap();
            prop_assert_eq!(taken_with.suit(), led);
            for card in cards.iter().filter(|card| card.suit() == led) {
                prop_assert!(*card <= taken_with, "{card} beats {taken_with}");
            }
            prop_assert_eq!(completed.leader(), leader);
            prop_assert_eq!(completed.cards(), cards.as_slice());
            prop_assert_eq!(trick, Trick::new(completed.taker()));
        }

        /// A held card can be played exactly when it's of the led suit, or the
        /// hand holds none of it.
        #[test]
        fn a_held_card_is_played_exactly_when_it_follows_suit(
            hand in hand_strategy(),
            trick in trick_strategy(),
            pick in any::<sample::Index>(),
        ) {
            let hand: Vec<_> = hand
                .into_iter()
                .filter(|held| trick.plays().all(|(_, played)| played != *held))
                .collect();
            prop_assume!(!hand.is_empty());
            let card = *pick.get(&hand);
            let required = trick
                .led_suit()
                .filter(|&suit| card.suit() != suit && hand.iter().any(|held| held.suit() == suit));
            let expected = match required {
                Some(required) => Err(Error::MustFollowSuit { card, required }),
                None => Ok(()),
            };
            let held = Held::new(card, &hand).expect("the card is in the hand");
            prop_assert_eq!(held.follow(&trick).map(|_| ()), expected);
        }
    }
}
