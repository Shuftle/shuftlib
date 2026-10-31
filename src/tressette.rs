//! Tressette for four players in two partnerships, played counter-clockwise.
//!
//! - Each seat must follow the led suit if it can; the strongest card of that
//!   suit takes the trick. From strongest: Three, Two, Ace, King, Knight, Jack,
//!   then Seven down to Four.
//! - A hand is worth eleven points: a point per three thirds taken (see
//!   [`TressetteCard::thirds`]), and one for the last trick.
//! - A team wins with at least [`SCORE_TO_WIN`] points and more than the other.
//! - Accusi ([`declaration`]) are declared on the seat's turn, before it plays,
//!   and count only for a team that scored from its tricks, the last one
//!   included.
//! - Signals ([`signal`]) are given only with a lead.
//! - When accusi may be made, what others learn from them, and which signals
//!   are allowed are set by the [`Config`].
//! - A seat that [forfeits](crate::engine::Session::forfeit) loses the game for
//!   its team.
//!
//! # Playing
//!
//! ```
//! # use std::error::Error;
//! use rand::{SeedableRng, rngs::StdRng, seq::IndexedRandom};
//! use shuftlib::{
//!     engine::{Observation, Session, Status},
//!     tressette::{
//!         Config, Tressette,
//!         declaration::{Declarations, Visibility, Window},
//!         signal::Signals,
//!     },
//! };
//!
//! // In a real game, the generator must be unpredictable, e.g. `rand::rng()`.
//! let mut rng = StdRng::seed_from_u64(42);
//! // Accusi on the first three tricks, told to everyone, and every signal allowed.
//! let config = Config {
//!     declarations: Declarations {
//!         window: Window::FirstThreeTricks,
//!         visibility: Visibility::Exact,
//!     },
//!     signals: Signals::all(),
//! };
//! let mut session = Session::<Tressette>::new(config);
//! let outcome = loop {
//!     match session.status() {
//!         Status::Chance => {
//!             session.resolve_chance(&mut rng)?;
//!         }
//!         // Tressette waits on one seat at a time, and it picks one of its
//!         // legal actions: a card, a declaration, or a lead with a signal.
//!         Status::ToAct(seats) => {
//!             let seat = seats[0];
//!             let legal = session.observe(seat).legal_actions();
//!             let action = *legal.choose(&mut rng).ok_or("no legal action")?;
//!             session.act(seat, action)?;
//!         }
//!         Status::Over(outcome) => break outcome,
//!     }
//! };
//! assert!(outcome.score[outcome.winner] >= 31);
//!
//! // The log holds every deal and every action: it replays on its own.
//! let replay = Session::<Tressette>::replay(config, session.log().to_vec())?;
//! assert_eq!(replay, session);
//! # Ok::<(), Box<dyn Error>>(())
//! ```

use std::collections::HashSet;

use rand::{Rng, RngExt};

use crate::{
    cards::{
        deck::Deck,
        italian::{PerSuit, Suit},
    },
    engine::{Events, Game, Observation, Status, Turn, Viewer, Waiting},
};

mod card;
pub mod declaration;
pub mod signal;
mod table;
mod trick;

pub use card::TressetteCard;
use declaration::{Declaration, DeclarationRefusal, Declared};
use signal::{Lead, Signal, SignalRefusal};
pub use table::{PerSeat, PerTeam, Seat, Team};
pub use trick::{CompletedTrick, Trick};
use trick::{Opening, Play};

/// The score a team needs to win, provided it's ahead.
pub const SCORE_TO_WIN: u8 = 31;

/// The points a team must score from its tricks in a hand for its
/// declarations to count.
const TRICK_POINTS_FOR_DECLARATIONS: u8 = 1;

/// The rules a game is played with. The default has declarations on the
/// first three tricks, with full visibility, and no signals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Config {
    /// When declarations may be made, and what other seats learn from them.
    pub declarations: declaration::Declarations,
    /// The signals a seat leading a trick may give.
    pub signals: signal::Signals,
}

/// The number of cards each seat is dealt.
const HAND_SIZE: usize = 10;

/// The dealer hands out this many cards at a time.
const DEAL_BATCH: usize = 5;

/// A game of Tressette. See the [module documentation](self) for the rules.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Tressette {
    config: Config,
    score: PerTeam<u8>,
    /// The dealer of the current or last hand. `None` until the first deal.
    dealer: Option<Seat>,
    phase: Phase,
}

/// Where a game stands.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Phase {
    /// Waiting for the next hand to be dealt.
    AwaitingDeal,
    /// A hand is being played. Boxed, as it's much larger than the others.
    Playing(Box<Hand>),
    /// The game is over.
    Over(Outcome),
}

impl Phase {
    /// Returns the hand being played, if any.
    fn hand(&self) -> Option<&Hand> {
        match self {
            Self::Playing(hand) => Some(hand),
            Self::AwaitingDeal | Self::Over(_) => None,
        }
    }
}

/// The chance outcome of Tressette: a deal.
///
/// The first dealer of a game is drawn at random, and the deal then passes
/// to the next seat, so only the first deal may have any `dealer`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Deal {
    /// The seat dealing.
    pub dealer: Seat,
    /// The cards each seat is dealt: ten each, the whole deck between them.
    pub hands: PerSeat<Vec<TressetteCard>>,
}

impl Deal {
    /// Shuffles the deck with `rng` and deals it from `dealer`, five cards at
    /// a time starting from the seat after them.
    pub fn shuffled(dealer: Seat, rng: &mut impl Rng) -> Self {
        let mut deck = deck();
        deck.shuffle(rng);
        Self::from_deck(dealer, deck)
    }

    /// Deals `deck` from `dealer`, five cards at a time from the top,
    /// starting from the seat after them.
    fn from_deck(dealer: Seat, mut deck: Deck<TressetteCard>) -> Self {
        let mut hands = PerSeat::<Vec<TressetteCard>>::default();
        let mut drawn = 0;
        while let Some(card) = deck.draw() {
            hands[dealer.after(drawn / DEAL_BATCH + 1)].push(card);
            drawn += 1;
        }
        Self { dealer, hands }
    }

    /// Returns whether the hands hold ten cards each, the whole deck between
    /// them.
    fn is_complete(&self) -> bool {
        let distinct: HashSet<TressetteCard> = Seat::ALL
            .iter()
            .flat_map(|&seat| self.hands[seat].iter().copied())
            .collect();
        distinct.len() == Seat::ALL.len() * HAND_SIZE
            && Seat::ALL
                .iter()
                .all(|&seat| self.hands[seat].len() == HAND_SIZE)
    }
}

/// Returns the 40 cards of the Italian deck, in deck order.
fn deck() -> Deck<TressetteCard> {
    Deck::italian()
        .into_iter()
        .map(TressetteCard::from)
        .collect()
}

/// A hand being played.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Hand {
    cards: PerSeat<Vec<TressetteCard>>,
    /// What each team took so far.
    taken: PerTeam<Vec<TressetteCard>>,
    trick: Trick,
    /// The last trick completed, if any.
    last_trick: Option<CompletedTrick>,
    /// What each seat declared this hand.
    declared: PerSeat<Vec<Declared>>,
    /// The suits each seat signalled in this hand.
    signalled: PerSeat<PerSuit<bool>>,
    /// The suits that opened a trick so far this hand.
    led: PerSuit<bool>,
}

impl Hand {
    /// Returns what a declaration by `seat` is read against, under `rules`.
    fn declaration_context(
        &self,
        rules: declaration::Declarations,
        seat: Seat,
    ) -> declaration::Context<'_> {
        declaration::Context {
            rules,
            trick: self.current_trick(),
            hand: &self.cards[seat],
            declared: &self.declared[seat],
        }
    }

    /// Returns what a signal by `seat` is read against, under `rules`.
    fn signal_context(&self, rules: signal::Signals, seat: Seat) -> signal::Context<'_> {
        signal::Context {
            rules,
            hand: &self.cards[seat],
            signalled: self.signalled[seat],
            led: self.led,
        }
    }

    /// Returns the 1-based number of the trick being played.
    fn current_trick(&self) -> usize {
        let taken: usize = Team::ALL.iter().map(|&team| self.taken[team].len()).sum();
        taken / Seat::ALL.len() + 1
    }

    /// Returns what each team scores for this hand, once every card has been
    /// played and `last_taker` took the last trick: its points from tricks,
    /// plus its declarations if those points are enough for them to count.
    fn points(&self, last_taker: Seat) -> PerTeam<u8> {
        PerTeam::from_fn(|team| {
            let thirds: u8 = self.taken[team].iter().map(TressetteCard::thirds).sum();
            let from_tricks = thirds / 3 + u8::from(team == last_taker.team());
            if from_tricks < TRICK_POINTS_FOR_DECLARATIONS {
                return from_tricks;
            }
            let declared: u8 = Seat::ALL
                .iter()
                .filter(|seat| seat.team() == team)
                .flat_map(|&seat| &self.declared[seat])
                .map(|made| made.declaration().points())
                .sum();
            from_tricks + declared
        })
    }
}

/// Something a seat can do on its turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    /// Plays a card from the hand.
    Play(TressetteCard),
    /// Declares an accuso or a napoli held in the hand, before playing. The
    /// seat keeps its turn.
    Declare(Declaration),
    /// Leads a trick with a card from the hand, giving a signal with it.
    Lead {
        /// The card led.
        card: TressetteCard,
        /// The signal given.
        signal: Signal,
    },
}

/// Something that happened in the game.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Event {
    /// A new hand was dealt.
    HandDealt {
        /// The seat that dealt.
        dealer: Seat,
    },
    /// The receiving seat was dealt these cards. Only that seat gets this
    /// event.
    CardsReceived {
        /// The cards dealt.
        cards: Vec<TressetteCard>,
    },
    /// A card was played.
    CardPlayed {
        /// The seat that played it.
        seat: Seat,
        /// The card played.
        card: TressetteCard,
    },
    /// A seat led the trick with a card, giving a signal with it.
    Led {
        /// The seat that led.
        seat: Seat,
        /// The card led.
        card: TressetteCard,
        /// The signal given.
        signal: Signal,
    },
    /// A seat made a declaration.
    Declared {
        /// The seat that declared.
        seat: Seat,
        /// The points the declaration is worth.
        points: u8,
        /// The declaration, or `None` when the rules keep it hidden.
        declaration: Option<Declaration>,
    },
    /// A trick was completed. Its taker leads the next one.
    TrickTaken {
        /// The completed trick.
        trick: CompletedTrick,
    },
    /// Every card of the hand was played and the hand was scored.
    HandScored {
        /// What each team scored in this hand.
        points: PerTeam<u8>,
        /// The score of the game, this hand included.
        score: PerTeam<u8>,
    },
    /// A seat forfeited, losing the game for its team.
    Forfeited {
        /// The seat that forfeited.
        seat: Seat,
    },
    /// The game is over.
    GameWon {
        /// The winning team.
        winner: Team,
    },
}

/// Why an action or a deal breaks the rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// No hand is being played. [`Session`](crate::engine::Session) only
    /// lets seats act during a hand, so it never returns this.
    #[error("no hand is being played")]
    NoHandInProgress,
    /// A hand is being played, or the game is over, so nothing can be dealt.
    /// [`Session`](crate::engine::Session) only applies deals between hands,
    /// so it never returns this.
    #[error("the game isn't waiting for a deal")]
    NotAwaitingDeal,
    /// The deal isn't by the seat whose turn it is to deal.
    #[error("the deal must be by {expected}")]
    WrongDealer {
        /// The seat whose turn it is to deal.
        expected: Seat,
    },
    /// The deal doesn't give ten cards to each seat, the whole deck between
    /// them.
    #[error("the deal must give ten cards to each seat, the whole deck between them")]
    IncompleteDeal,
    /// The card isn't in the acting seat's hand.
    #[error("{card} is not in the hand")]
    CardNotInHand {
        /// The card played.
        card: TressetteCard,
    },
    /// The seat tried to lead, but the trick already has a card.
    #[error("the trick has already been led")]
    NotLeading,
    /// The card doesn't follow the led suit, and the hand has cards of it.
    #[error("{card} was played, but the hand must follow {required}")]
    MustFollowSuit {
        /// The card played.
        card: TressetteCard,
        /// The suit that led the trick.
        required: Suit,
    },
    /// The declaration can't be made now.
    #[error("{declaration} can't be declared: {reason}")]
    CannotDeclare {
        /// The declaration attempted.
        declaration: Declaration,
        /// Why it can't be made.
        reason: DeclarationRefusal,
    },
    /// The signal can't be given with this lead.
    #[error("{signal:?} can't be given: {reason}")]
    CannotSignal {
        /// The signal given.
        signal: Signal,
        /// Why it can't be given.
        reason: SignalRefusal,
    },
}

/// The result of a finished game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Outcome {
    /// The winning team.
    pub winner: Team,
    /// The final score. It may fall short of [`SCORE_TO_WIN`] if the game
    /// ended because a seat forfeited.
    pub score: PerTeam<u8>,
    /// The seat that forfeited, if that's how the game ended.
    pub forfeited: Option<Seat>,
}

/// What a seat is allowed to know about the game: its own hand, and
/// everything on the table. A spectator sees the table only.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct View {
    viewer: Viewer<Seat>,
    status: Status<Seat, Outcome>,
    hand: Vec<TressetteCard>,
    hand_sizes: PerSeat<usize>,
    trick: Option<Trick>,
    last_trick: Option<CompletedTrick>,
    tricks_taken: PerTeam<usize>,
    score: PerTeam<u8>,
    dealer: Option<Seat>,
    declarable: Vec<Declaration>,
    leads: Vec<(TressetteCard, Signal)>,
}

impl View {
    /// Returns who is observing.
    pub fn viewer(&self) -> Viewer<Seat> {
        self.viewer
    }

    /// Returns the observing seat's hand. Empty for a spectator.
    pub fn hand(&self) -> &[TressetteCard] {
        &self.hand
    }

    /// Returns how many cards each seat holds.
    pub fn hand_sizes(&self) -> PerSeat<usize> {
        self.hand_sizes
    }

    /// Returns the trick being played, or `None` between hands.
    pub fn trick(&self) -> Option<&Trick> {
        self.trick.as_ref()
    }

    /// Returns the last trick completed in this hand, which the rules let
    /// anyone look back at, or `None` before the first one and between
    /// hands.
    pub fn last_trick(&self) -> Option<&CompletedTrick> {
        self.last_trick.as_ref()
    }

    /// Returns how many tricks each team has taken in this hand.
    pub fn tricks_taken(&self) -> PerTeam<usize> {
        self.tricks_taken
    }

    /// Returns the score of the game.
    pub fn score(&self) -> PerTeam<u8> {
        self.score
    }

    /// Returns the seat that dealt the current or last hand, or `None`
    /// before the first deal.
    pub fn dealer(&self) -> Option<Seat> {
        self.dealer
    }

    /// Returns the cards the observing seat may play. Empty when it's not
    /// that seat's turn, and for a spectator.
    pub fn legal_cards(&self) -> Vec<TressetteCard> {
        match (&self.trick, self.viewer.seat()) {
            (Some(trick), Some(seat)) if self.status.to_act().contains(&seat) => self
                .hand
                .iter()
                .copied()
                .filter(|&card| Play::new(card, &self.hand, trick).is_ok())
                .collect(),
            _ => Vec::new(),
        }
    }

    /// Returns the declarations the observing seat may make now. Empty when
    /// it's not that seat's turn, outside the tricks the rules allow, and for
    /// a spectator.
    pub fn declarable(&self) -> &[Declaration] {
        &self.declarable
    }

    /// Returns the cards the observing seat may lead with a signal now, each
    /// with the signal. Empty when it isn't leading the trick, and for a
    /// spectator.
    pub fn leads(&self) -> &[(TressetteCard, Signal)] {
        &self.leads
    }
}

impl Observation for View {
    type Seat = Seat;
    type Action = Action;
    type Outcome = Outcome;
    type Legal = Vec<Action>;

    fn status(&self) -> Status<Seat, Outcome> {
        self.status.clone()
    }

    fn legal_actions(&self) -> Vec<Action> {
        self.legal_cards()
            .into_iter()
            .map(Action::Play)
            .chain(self.declarable.iter().copied().map(Action::Declare))
            .chain(
                self.leads
                    .iter()
                    .map(|&(card, signal)| Action::Lead { card, signal }),
            )
            .collect()
    }
}

impl Tressette {
    /// Returns the team that has won with the score, if any: the one with at
    /// least [`SCORE_TO_WIN`] points and more than the other.
    fn winner(&self) -> Option<Team> {
        Team::ALL.into_iter().find(|&team| {
            self.score[team] >= SCORE_TO_WIN && self.score[team] > self.score[team.opponent()]
        })
    }

    /// Adds the `points` of the hand that just ended to the score, and either
    /// ends the game or waits for the next deal.
    fn score_hand(&mut self, points: PerTeam<u8>, events: &mut Events<Seat, Event>) {
        // The game ends as soon as a team at 31 or more is ahead, so scores
        // only grow past that while the teams stay tied, which with
        // declarations can happen. Reaching 255 would take a dozen tied hands
        // in a row, so the score saturates there rather than wrapping.
        for team in Team::ALL {
            self.score[team] = self.score[team].saturating_add(points[team]);
        }
        events.public(Event::HandScored {
            points,
            score: self.score,
        });
        match self.winner() {
            Some(winner) => self.end(winner, None, events),
            None => self.phase = Phase::AwaitingDeal,
        }
    }

    /// Ends the game with `winner` winning, by forfeit of `forfeited` if it's
    /// set.
    fn end(&mut self, winner: Team, forfeited: Option<Seat>, events: &mut Events<Seat, Event>) {
        events.public(Event::GameWon { winner });
        self.phase = Phase::Over(Outcome {
            winner,
            score: self.score,
            forfeited,
        });
    }
}

impl Game for Tressette {
    type Config = Config;
    type Seat = Seat;
    type Action = Action;
    type Chance = Deal;
    type Event = Event;
    type Observation = View;
    type Outcome = Outcome;
    type Error = Error;

    fn seats(&self) -> &[Seat] {
        &Seat::ALL
    }

    fn new(config: &Config) -> Self {
        Self {
            config: *config,
            score: PerTeam::default(),
            dealer: None,
            phase: Phase::AwaitingDeal,
        }
    }

    fn status(&self) -> Status<Seat, Outcome> {
        match &self.phase {
            Phase::AwaitingDeal => Status::Chance,
            Phase::Playing(hand) => Status::ToAct(Waiting::one(hand.trick.next_to_play())),
            Phase::Over(outcome) => Status::Over(*outcome),
        }
    }

    fn act(&mut self, turn: Turn<Seat>, action: Action) -> Result<Events<Seat, Event>, Error> {
        let Phase::Playing(hand) = &mut self.phase else {
            return Err(Error::NoHandInProgress);
        };
        let seat = turn.seat();
        let (play, lead) = match action {
            Action::Play(card) => (Play::new(card, &hand.cards[seat], &hand.trick)?, None),
            Action::Declare(declaration) => {
                let rules = self.config.declarations;
                let made = Declared::new(declaration, &hand.declaration_context(rules, seat))?;
                hand.declared[seat].push(made);
                let mut events = Events::new();
                events.public(Event::Declared {
                    seat,
                    points: declaration.points(),
                    declaration: (rules.visibility == declaration::Visibility::Exact)
                        .then_some(declaration),
                });
                return Ok(events);
            }
            Action::Lead { card, signal } => {
                let opening = Opening::new(card, &hand.cards[seat], &hand.trick)?;
                let lead = Lead::new(
                    opening,
                    signal,
                    &hand.signal_context(self.config.signals, seat),
                )?;
                hand.signalled[seat][lead.card().suit()] = true;
                (lead.play(), Some(lead))
            }
        };
        let card = play.card();
        if hand.trick.led_suit().is_none() {
            hand.led[card.suit()] = true;
        }
        hand.cards[seat].retain(|&held| held != card);
        let mut events = Events::new();
        let completed = match lead {
            Some(lead) => {
                events.public(Event::Led {
                    seat,
                    card,
                    signal: lead.signal(),
                });
                hand.trick.lead(lead)
            }
            None => {
                events.public(Event::CardPlayed { seat, card });
                hand.trick.play(play)
            }
        };
        if let Some(trick) = completed {
            hand.taken[trick.taker().team()].extend(trick.cards());
            hand.last_trick = Some(trick);
            events.public(Event::TrickTaken { trick });
            if Seat::ALL.iter().all(|&seat| hand.cards[seat].is_empty()) {
                let points = hand.points(trick.taker());
                self.score_hand(points, &mut events);
            }
        }
        Ok(events)
    }

    fn sample_chance(&self, rng: &mut impl Rng) -> Option<Deal> {
        if !matches!(self.phase, Phase::AwaitingDeal) {
            return None;
        }
        let dealer = match self.dealer {
            Some(previous) => previous.next(),
            None => Seat::ALL[rng.random_range(0..Seat::ALL.len())],
        };
        Some(Deal::shuffled(dealer, rng))
    }

    fn apply_chance(&mut self, deal: Deal) -> Result<Events<Seat, Event>, Error> {
        if !matches!(self.phase, Phase::AwaitingDeal) {
            return Err(Error::NotAwaitingDeal);
        }
        if let Some(expected) = self
            .dealer
            .map(Seat::next)
            .filter(|&expected| expected != deal.dealer)
        {
            return Err(Error::WrongDealer { expected });
        }
        if !deal.is_complete() {
            return Err(Error::IncompleteDeal);
        }

        let Deal { dealer, hands } = deal;
        let mut events = Events::new();
        events.public(Event::HandDealt { dealer });
        for seat in Seat::ALL {
            events.private(
                seat,
                Event::CardsReceived {
                    cards: hands[seat].clone(),
                },
            );
        }
        self.dealer = Some(dealer);
        self.phase = Phase::Playing(Box::new(Hand {
            cards: hands,
            taken: PerTeam::default(),
            trick: Trick::new(dealer.next()),
            last_trick: None,
            declared: PerSeat::default(),
            signalled: PerSeat::default(),
            led: PerSuit::default(),
        }));
        Ok(events)
    }

    fn forfeit(&mut self, turn: Turn<Seat>) -> Events<Seat, Event> {
        let seat = turn.seat();
        let mut events = Events::new();
        events.public(Event::Forfeited { seat });
        self.end(seat.team().opponent(), Some(seat), &mut events);
        events
    }

    fn observe(&self, viewer: Viewer<Seat>) -> View {
        let hand = self.phase.hand();
        let to_act = hand
            .zip(viewer.seat())
            .filter(|(hand, seat)| hand.trick.next_to_play() == *seat);
        View {
            viewer,
            status: self.status(),
            hand: hand
                .zip(viewer.seat())
                .map_or(Vec::new(), |(hand, seat)| hand.cards[seat].clone()),
            hand_sizes: PerSeat::from_fn(|seat| hand.map_or(0, |hand| hand.cards[seat].len())),
            trick: hand.map(|hand| hand.trick),
            last_trick: hand.and_then(|hand| hand.last_trick),
            tricks_taken: PerTeam::from_fn(|team| {
                hand.map_or(0, |hand| hand.taken[team].len() / Seat::ALL.len())
            }),
            score: self.score,
            dealer: self.dealer,
            declarable: to_act.map_or(Vec::new(), |(hand, seat)| {
                declaration::allowed(&hand.declaration_context(self.config.declarations, seat))
            }),
            leads: to_act.map_or(Vec::new(), |(hand, seat)| {
                signal::allowed(&hand.signal_context(self.config.signals, seat), &hand.trick)
                    .into_iter()
                    .map(|lead| (lead.card(), lead.signal()))
                    .collect()
            }),
        }
    }
}

#[cfg(test)]
/// Test utilities for the Tressette module.
pub mod test_utils {
    use super::{
        Config, HAND_SIZE, Seat, TressetteCard,
        card::test_utils::cards_of_size,
        declaration::{DeclarableRank, test_utils::rules_strategy},
    };
    use crate::cards::italian::Suit;
    use proptest::prelude::*;

    /// The most actions a hand can take: every card played, and every seat
    /// making every declaration it can, a napoli per suit and an accuso per
    /// rank.
    pub const MAX_HAND_ACTIONS: usize =
        Seat::ALL.len() * (HAND_SIZE + Suit::ALL.len() + DeclarableRank::ALL.len());

    /// Generates any game configuration.
    pub fn config_strategy() -> impl Strategy<Value = Config> {
        (
            rules_strategy(),
            super::signal::test_utils::rules_strategy(),
        )
            .prop_map(|(declarations, signals)| Config {
                declarations,
                signals,
            })
    }

    /// Generates a hand a seat could hold: 1 to [`HAND_SIZE`] distinct cards.
    pub fn hand_strategy() -> impl Strategy<Value = Vec<TressetteCard>> {
        cards_of_size(1..=HAND_SIZE)
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use rand::{SeedableRng, rngs::StdRng};

    use super::*;
    use crate::{
        cards::italian::ItalianRank,
        engine::{ActError, ActResult, Audience, ChanceError, Session, UpdateOf},
        tressette::{
            card::test_utils::cards_strategy,
            declaration::{
                Visibility,
                test_utils::{declared_by_seat_strategy, made},
            },
            table::test_utils::seat_strategy,
            test_utils::{MAX_HAND_ACTIONS, config_strategy},
        },
    };

    fn suit(suit: Suit) -> Vec<TressetteCard> {
        ItalianRank::ALL
            .iter()
            .map(|&rank| TressetteCard::new(rank, suit))
            .collect()
    }

    fn card(rank: ItalianRank, suit: Suit) -> TressetteCard {
        TressetteCard::new(rank, suit)
    }

    /// Each seat holds a whole suit.
    fn by_suit() -> PerSeat<Vec<TressetteCard>> {
        PerSeat {
            north: suit(Suit::Coins),
            east: suit(Suit::Cups),
            south: suit(Suit::Swords),
            west: suit(Suit::Clubs),
        }
    }

    /// A hand in progress, dealt by North, with the given hands.
    fn playing(cards: PerSeat<Vec<TressetteCard>>) -> Session<Tressette> {
        Session::from_game(
            Config::default(),
            Tressette {
                config: Config::default(),
                score: PerTeam::default(),
                dealer: Some(Seat::North),
                phase: Phase::Playing(Box::new(Hand {
                    cards,
                    taken: PerTeam::default(),
                    trick: Trick::new(Seat::North.next()),
                    last_trick: None,
                    declared: PerSeat::default(),
                    signalled: PerSeat::default(),
                    led: PerSuit::default(),
                })),
            },
        )
    }

    /// Returns the seat the game is waiting on, checking there's only one.
    fn to_act(session: &Session<Tressette>) -> Option<Seat> {
        let status = session.status();
        assert!(
            status.to_act().len() <= 1,
            "{status:?} waits on several seats"
        );
        status.to_act().first().copied()
    }

    fn play(
        session: &mut Session<Tressette>,
        seat: Seat,
        card: TressetteCard,
    ) -> ActResult<Tressette> {
        session.act(seat, Action::Play(card))
    }

    /// Plays the first legal action until the hand is over, and returns the
    /// public events of every action.
    fn play_hand(session: &mut Session<Tressette>) -> Vec<Event> {
        let mut events = Vec::new();
        let mut actions = 0;
        while let Some(seat) = to_act(session) {
            actions += 1;
            assert!(actions <= MAX_HAND_ACTIONS, "the hand never ends");
            let action = session.observe(seat).legal_actions()[0];
            events.extend(session.act(seat, action).unwrap().public_only().cloned());
        }
        events
    }

    #[test]
    fn the_led_suit_must_be_followed_even_after_a_discard() {
        let mut session = playing(PerSeat {
            west: vec![card(ItalianRank::King, Suit::Cups)],
            south: vec![card(ItalianRank::Five, Suit::Coins)],
            east: vec![
                card(ItalianRank::Six, Suit::Cups),
                card(ItalianRank::Seven, Suit::Coins),
            ],
            north: vec![card(ItalianRank::Four, Suit::Cups)],
        });
        play(
            &mut session,
            Seat::West,
            card(ItalianRank::King, Suit::Cups),
        )
        .unwrap();
        // South can't follow and discards a Coin: East must still follow Cups.
        play(
            &mut session,
            Seat::South,
            card(ItalianRank::Five, Suit::Coins),
        )
        .unwrap();

        let view = session.observe(Seat::East);
        assert_eq!(
            view.legal_actions(),
            [Action::Play(card(ItalianRank::Six, Suit::Cups))]
        );
        let before = session.clone();
        assert_eq!(
            play(
                &mut session,
                Seat::East,
                card(ItalianRank::Seven, Suit::Coins)
            ),
            Err(ActError::Illegal(Error::MustFollowSuit {
                card: card(ItalianRank::Seven, Suit::Coins),
                required: Suit::Cups,
            }))
        );
        assert_eq!(session, before);
        play(&mut session, Seat::East, card(ItalianRank::Six, Suit::Cups)).unwrap();
    }

    #[test]
    fn cards_not_in_hand_are_rejected_before_suit_is_checked() {
        let mut session = playing(by_suit());
        play(
            &mut session,
            Seat::West,
            card(ItalianRank::Ace, Suit::Clubs),
        )
        .unwrap();
        let before = session.clone();
        // South holds only Swords: a Coin is both foreign and off suit.
        let foreign = card(ItalianRank::Ace, Suit::Coins);
        assert_eq!(
            play(&mut session, Seat::South, foreign),
            Err(ActError::Illegal(Error::CardNotInHand { card: foreign }))
        );
        assert_eq!(session, before);
    }

    /// Returns the team that has won with `score`, if any.
    fn winner(score: PerTeam<u8>) -> Option<Team> {
        Tressette {
            score,
            ..Tressette::new(&Config::default())
        }
        .winner()
    }

    proptest! {
        /// Whatever the score, at most one team has won, a winner has reached
        /// the target and is ahead, and below the target or on a tie the game
        /// goes on. Scores never pass 52, so that's where they're drawn.
        #[test]
        fn only_a_team_ahead_at_the_target_wins(north_south in 0u8..=60, east_west in 0u8..=60) {
            let score = PerTeam { north_south, east_west };
            match winner(score) {
                Some(team) => {
                    prop_assert!(score[team] >= SCORE_TO_WIN);
                    prop_assert!(score[team] > score[team.opponent()]);
                }
                None => prop_assert!(
                    north_south == east_west
                        || north_south.max(east_west) < SCORE_TO_WIN
                ),
            }
        }

        /// However the deck ends up split between the teams, a hand is worth
        /// eleven points, and the team that took the last trick scores at
        /// least the point for it, even when it took nothing else.
        #[test]
        fn every_hand_is_worth_eleven_points(
            north_south_takes in cards_strategy(),
            last_taker in seat_strategy(),
        ) {
            let hand = split_deck(&north_south_takes, last_taker);
            let points = hand.points(last_taker);
            prop_assert_eq!(points.north_south + points.east_west, 11);
            prop_assert!(points[last_taker.team()] >= 1);
        }

        /// Declarations add their points to a team that scored at least a
        /// point from its tricks, and nothing to a team that didn't.
        #[test]
        fn declarations_count_only_for_a_team_that_scored_from_its_tricks(
            north_south_takes in cards_strategy(),
            last_taker in seat_strategy(),
            declared in declared_by_seat_strategy(),
        ) {
            let mut hand = split_deck(&north_south_takes, last_taker);
            let from_tricks = hand.points(last_taker);
            hand.declared = PerSeat::from_fn(|seat| {
                declared[seat].iter().copied().map(made).collect()
            });
            let points = hand.points(last_taker);
            for team in Team::ALL {
                let declared: u8 = Seat::ALL
                    .iter()
                    .filter(|seat| seat.team() == team)
                    .flat_map(|&seat| &declared[seat])
                    .map(Declaration::points)
                    .sum();
                let expected = if from_tricks[team] >= TRICK_POINTS_FOR_DECLARATIONS {
                    from_tricks[team] + declared
                } else {
                    from_tricks[team]
                };
                prop_assert_eq!(points[team], expected);
            }
        }

        /// A deal with a card moved to another hand, dropped, or replaced by
        /// a copy of another is rejected, and the game is left unchanged.
        #[test]
        fn incomplete_deals_are_rejected(
            seed: u64,
            from in seat_strategy(),
            to in seat_strategy(),
            card in 0usize..HAND_SIZE,
            other in 0usize..HAND_SIZE,
            mistake in 0u8..3,
        ) {
            let mut deal = Deal::shuffled(Seat::North, &mut StdRng::seed_from_u64(seed));
            match mistake {
                0 => {
                    prop_assume!(from != to);
                    let moved = deal.hands[from].remove(card);
                    deal.hands[to].push(moved);
                }
                1 => {
                    deal.hands[from].remove(card);
                }
                _ => {
                    prop_assume!(from != to || card != other);
                    deal.hands[from][card] = deal.hands[to][other];
                }
            }
            let mut session = Session::<Tressette>::new(Config::default());
            let before = session.clone();
            prop_assert_eq!(
                session.resolve_chance_with(deal),
                Err(ChanceError::Illegal(Error::IncompleteDeal))
            );
            prop_assert_eq!(session, before);
        }
    }

    /// A finished hand: North and South took `north_south_takes`, East and
    /// West the rest of the deck, and `last_taker` took the last trick.
    /// Nobody declared anything.
    fn split_deck(north_south_takes: &[TressetteCard], last_taker: Seat) -> Hand {
        let east_west_takes = deck()
            .into_iter()
            .filter(|card| !north_south_takes.contains(card))
            .collect();
        let taken = PerTeam {
            north_south: north_south_takes.to_vec(),
            east_west: east_west_takes,
        };
        Hand {
            cards: PerSeat::default(),
            taken,
            trick: Trick::new(last_taker),
            last_trick: None,
            declared: PerSeat::default(),
            signalled: PerSeat::default(),
            led: PerSuit::default(),
        }
    }

    #[test]
    fn drawn_deals_pass_on_and_the_next_seat_leads() {
        let mut rng = StdRng::seed_from_u64(9);
        let mut session = Session::<Tressette>::new(Config::default());
        session.resolve_chance(&mut rng).unwrap();
        let first = session.observe(Seat::North).dealer().unwrap();
        assert_eq!(to_act(&session), Some(first.next()));
        play_hand(&mut session);

        let update = session.resolve_chance(&mut rng).unwrap();
        let second = first.next();
        assert_eq!(
            update.public_only().collect::<Vec<_>>(),
            [&Event::HandDealt { dealer: second }]
        );
        assert_eq!(to_act(&session), Some(second.next()));
    }

    #[test]
    fn forfeiting_loses_the_game_for_the_team() {
        let mut session = playing(by_suit());
        // West is to play, but any seat may forfeit.
        assert_eq!(to_act(&session), Some(Seat::West));
        let update = session.forfeit(Seat::South).unwrap();
        assert_eq!(
            update.public_only().collect::<Vec<_>>(),
            [
                &Event::Forfeited { seat: Seat::South },
                &Event::GameWon {
                    winner: Team::EastWest
                },
            ]
        );
        assert_eq!(
            session.status(),
            Status::Over(Outcome {
                winner: Team::EastWest,
                score: PerTeam::default(),
                forfeited: Some(Seat::South),
            })
        );

        // Between hands too.
        let mut session = Session::<Tressette>::new(Config::default());
        session.forfeit(Seat::East).unwrap();
        assert!(matches!(
            session.status(),
            Status::Over(Outcome {
                winner: Team::NorthSouth,
                ..
            })
        ));
    }

    #[test]
    fn deals_can_be_given_and_are_checked() {
        let deal = |dealer| Deal {
            dealer,
            hands: by_suit(),
        };
        let mut session = Session::<Tressette>::new(Config::default());
        session.resolve_chance_with(deal(Seat::North)).unwrap();
        assert_eq!(session.observe(Seat::South).hand(), suit(Suit::Swords));
        assert_eq!(to_act(&session), Some(Seat::West));
        play_hand(&mut session);

        // The deal passes on from North to West.
        let before = session.clone();
        assert_eq!(
            session.resolve_chance_with(deal(Seat::South)),
            Err(ChanceError::Illegal(Error::WrongDealer {
                expected: Seat::West
            }))
        );
        assert_eq!(session, before);
        session.resolve_chance_with(deal(Seat::West)).unwrap();
    }

    #[test]
    fn the_deck_is_dealt_in_fives_from_the_seat_after_the_dealer() {
        let cards = |suit, ranks: [ItalianRank; 5]| ranks.map(|rank| card(rank, suit));
        let high = [
            ItalianRank::King,
            ItalianRank::Knight,
            ItalianRank::Jack,
            ItalianRank::Seven,
            ItalianRank::Six,
        ];
        let low = [
            ItalianRank::Five,
            ItalianRank::Four,
            ItalianRank::Three,
            ItalianRank::Two,
            ItalianRank::Ace,
        ];
        // The unshuffled deck has the King of Clubs on top. East deals: North
        // gets the top five cards, then West, South and East the next five
        // each, and round again.
        let deal = Deal::from_deck(Seat::East, deck());
        let hand = |first: [TressetteCard; 5], second: [TressetteCard; 5]| {
            first.into_iter().chain(second).collect::<Vec<_>>()
        };
        assert_eq!(
            deal.hands,
            PerSeat {
                north: hand(cards(Suit::Clubs, high), cards(Suit::Cups, high)),
                west: hand(cards(Suit::Clubs, low), cards(Suit::Cups, low)),
                south: hand(cards(Suit::Swords, high), cards(Suit::Coins, high)),
                east: hand(cards(Suit::Swords, low), cards(Suit::Coins, low)),
            }
        );
    }

    /// Tricks are decided by comparing cards, so the order must be a total
    /// order that agrees with equality.
    #[test]
    fn cards_are_ordered_consistently_with_equality() {
        let cards: Vec<_> = deck().into_iter().collect();
        for &a in &cards {
            for &b in &cards {
                assert_eq!(
                    a.cmp(&b) == std::cmp::Ordering::Equal,
                    a == b,
                    "{a} and {b}"
                );
                assert_eq!(a.cmp(&b), b.cmp(&a).reverse(), "{a} and {b}");
                for &c in &cards {
                    if a < b && b < c {
                        assert!(a < c, "{a} < {b} < {c}");
                    }
                }
            }
        }
    }

    /// What one seat knows, rebuilt from nothing but the events it receives.
    #[derive(Debug, Default)]
    struct Tracker {
        hand: Vec<TressetteCard>,
        hand_sizes: PerSeat<usize>,
        trick: Vec<(Seat, TressetteCard)>,
        signal: Option<Signal>,
        last_trick: Option<CompletedTrick>,
        tricks_taken: PerTeam<usize>,
        score: PerTeam<u8>,
        dealer: Option<Seat>,
    }

    impl Tracker {
        fn apply(&mut self, viewer: Option<Seat>, event: &Event) {
            match event {
                Event::HandDealt { dealer } => {
                    *self = Self {
                        hand_sizes: PerSeat::from_fn(|_| HAND_SIZE),
                        score: self.score,
                        dealer: Some(*dealer),
                        ..Self::default()
                    };
                }
                Event::CardsReceived { cards } => self.hand.clone_from(cards),
                Event::CardPlayed { seat: player, card }
                | Event::Led {
                    seat: player, card, ..
                } => {
                    if Some(*player) == viewer {
                        let position = self.hand.iter().position(|held| held == card).unwrap();
                        self.hand.remove(position);
                    }
                    self.hand_sizes[*player] -= 1;
                    self.trick.push((*player, *card));
                    if let Event::Led { signal, .. } = event {
                        self.signal = Some(*signal);
                    }
                }
                Event::TrickTaken { trick } => {
                    assert_eq!(trick.plays().collect::<Vec<_>>(), self.trick);
                    assert_eq!(trick.signal(), self.signal);
                    self.trick.clear();
                    self.signal = None;
                    self.tricks_taken[trick.taker().team()] += 1;
                    self.last_trick = Some(*trick);
                }
                Event::HandScored { score, .. } => {
                    *self = Self {
                        score: *score,
                        dealer: self.dealer,
                        ..Self::default()
                    };
                }
                // The hand is abandoned.
                Event::Forfeited { .. } => {
                    *self = Self {
                        score: self.score,
                        dealer: self.dealer,
                        ..Self::default()
                    };
                }
                Event::Declared { .. } | Event::GameWon { .. } => {}
            }
        }

        fn assert_matches(&self, view: &View) {
            assert_eq!(view.hand(), self.hand);
            assert_eq!(view.hand_sizes(), self.hand_sizes);
            assert_eq!(view.last_trick(), self.last_trick.as_ref());
            assert_eq!(view.tricks_taken(), self.tricks_taken);
            assert_eq!(view.score(), self.score);
            assert_eq!(view.dealer(), self.dealer);
            let trick = view
                .trick()
                .map_or(Vec::new(), |trick| trick.plays().collect());
            assert_eq!(trick, self.trick);
            assert_eq!(view.trick().and_then(Trick::signal), self.signal);
        }
    }

    /// Every card of a hand in progress is in exactly one place: a hand, a
    /// team's pile or the trick.
    fn assert_cards_conserved(hand: &Hand) {
        let places: Vec<TressetteCard> = Seat::ALL
            .iter()
            .flat_map(|&seat| hand.cards[seat].iter().copied())
            .chain(
                Team::ALL
                    .iter()
                    .flat_map(|&team| hand.taken[team].iter().copied()),
            )
            .chain(hand.trick.plays().map(|(_, card)| card))
            .collect();
        let distinct: HashSet<_> = places.iter().copied().collect();
        assert_eq!(places.len(), 40);
        assert_eq!(distinct.len(), 40);
    }

    /// Checks that the actions `seat` may take are exactly the ones the game
    /// accepts, and that rejected ones leave it unchanged.
    fn assert_legality_is_exact(session: &Session<Tressette>, seat: Seat) {
        let legal = session.observe(seat).legal_actions();
        let leads = deck().into_iter().flat_map(|card| {
            Signal::ALL
                .into_iter()
                .map(move |signal| Action::Lead { card, signal })
        });
        let actions = deck()
            .into_iter()
            .map(Action::Play)
            .chain(Declaration::all().map(Action::Declare))
            .chain(leads);
        for action in actions {
            let mut attempt = session.clone();
            match attempt.act(seat, action) {
                Ok(_) => assert!(legal.contains(&action), "{action:?} was accepted"),
                Err(ActError::Illegal(_)) => {
                    assert!(!legal.contains(&action), "{action:?} was rejected");
                    assert_eq!(&attempt, session);
                }
                Err(error) => panic!("unexpected error {error:?}"),
            }
        }
        for other in Seat::ALL.into_iter().filter(|&other| other != seat) {
            assert!(session.observe(other).legal_actions().is_empty());
        }
    }

    /// Checks that `seat` is offered exactly the declarations it may still
    /// make, and only on the tricks `config` allows.
    fn assert_declarable_is_exact(session: &Session<Tressette>, config: Config, seat: Seat) {
        let hand = session.game().phase.hand().unwrap();
        // The seat to act has played one card per completed trick.
        let trick = HAND_SIZE - hand.cards[seat].len() + 1;
        let expected = declaration::allowed(&declaration::Context {
            rules: config.declarations,
            trick,
            hand: &hand.cards[seat],
            declared: &hand.declared[seat],
        });
        assert_eq!(session.observe(seat).declarable(), expected);
    }

    /// Checks that `seat` is offered a signalled lead exactly when it leads the
    /// trick with a signal that's allowed and true, given the suits it already
    /// `signalled` and the suits already `led`.
    fn assert_signals_are_exact(
        session: &Session<Tressette>,
        config: Config,
        seat: Seat,
        signalled: PerSuit<bool>,
        led: PerSuit<bool>,
    ) {
        let hand = session.game().phase.hand().unwrap();
        let held = &hand.cards[seat];
        let context = signal::Context {
            rules: config.signals,
            hand: held,
            signalled,
            led,
        };
        let leading = hand.trick.led_suit().is_none();
        let expected: HashSet<_> = held
            .iter()
            .flat_map(|&card| Signal::ALL.map(|signal| (card, signal)))
            .filter(|&(card, signal)| {
                leading
                    && Opening::new(card, held, &hand.trick)
                        .and_then(|opening| Lead::new(opening, signal, &context))
                        .is_ok()
            })
            .collect();
        let view = session.observe(seat);
        let offered: HashSet<_> = view
            .legal_actions()
            .into_iter()
            .filter_map(|action| match action {
                Action::Lead { card, signal } => Some((card, signal)),
                _ => None,
            })
            .collect();
        assert_eq!(offered, expected);
        assert_eq!(
            view.leads().iter().copied().collect::<HashSet<_>>(),
            expected
        );
    }

    /// Checks that `action` by `seat` told everyone which card was played in
    /// a single event, with its signal when it led with one.
    fn assert_card_event(seat: Seat, action: Action, update: &UpdateOf<Tressette>) {
        let cards: Vec<_> = update
            .public_only()
            .filter(|event| matches!(event, Event::CardPlayed { .. } | Event::Led { .. }))
            .collect();
        match action {
            Action::Play(card) => assert_eq!(cards, [&Event::CardPlayed { seat, card }]),
            Action::Lead { card, signal } => {
                assert_eq!(cards, [&Event::Led { seat, card, signal }]);
            }
            Action::Declare(_) => assert!(cards.is_empty()),
        }
    }

    /// Checks that declaring `declaration` told everyone its points, and
    /// which one it was only when `config` says so, and that `seat` kept the
    /// turn.
    fn assert_declared(
        session: &Session<Tressette>,
        config: Config,
        seat: Seat,
        declaration: Declaration,
        update: &UpdateOf<Tressette>,
    ) {
        let shown = config.declarations.visibility == Visibility::Exact;
        assert_eq!(
            update.public_only().collect::<Vec<_>>(),
            [&Event::Declared {
                seat,
                points: declaration.points(),
                declaration: shown.then_some(declaration),
            }]
        );
        assert_eq!(to_act(session), Some(seat));
    }

    /// Checks that a hand is worth eleven points plus the declarations of
    /// every team that scored from its tricks: a team that didn't scores
    /// nothing at all.
    fn assert_hand_points(points: PerTeam<u8>, declared: PerTeam<u8>) {
        let counted: u8 = Team::ALL
            .iter()
            .filter(|&&team| points[team] > 0)
            .map(|&team| declared[team])
            .sum();
        assert_eq!(points.north_south + points.east_west, 11 + counted);
        for team in Team::ALL {
            assert!(points[team] == 0 || points[team] > declared[team]);
        }
    }

    /// Plays a whole game under `config` with deals drawn from `deals` and
    /// random legal actions from `choices`, plus the odd forfeit, checking
    /// invariants along the way, and returns it.
    fn play_out(config: Config, deals: u64, choices: u64) -> Session<Tressette> {
        let mut deals = StdRng::seed_from_u64(deals);
        let mut choices = StdRng::seed_from_u64(choices);
        let mut session = Session::<Tressette>::new(config);
        let mut trackers = PerSeat::<Tracker>::default();
        let mut spectator = Tracker::default();
        let mut declared = PerTeam::<u8>::default();
        let mut hand_actions = 0;
        let mut signalled = PerSeat::<PerSuit<bool>>::default();
        let mut led = PerSuit::<bool>::default();
        loop {
            let update = match session.status() {
                Status::Chance => session.resolve_chance(&mut deals).unwrap(),
                Status::ToAct(seats) => {
                    let [seat] = seats[..] else {
                        panic!("{seats:?} are waited on together");
                    };
                    assert_legality_is_exact(&session, seat);
                    assert_declarable_is_exact(&session, config, seat);
                    assert_signals_are_exact(&session, config, seat, signalled[seat], led);
                    if choices.random_ratio(1, 500) {
                        let quitter = Seat::ALL[choices.random_range(0..Seat::ALL.len())];
                        session.forfeit(quitter).unwrap()
                    } else {
                        hand_actions += 1;
                        assert!(hand_actions <= MAX_HAND_ACTIONS, "the hand never ends");
                        let legal = session.observe(seat).legal_actions();
                        let action = legal[choices.random_range(0..legal.len())];
                        let leading = session
                            .observe(seat)
                            .trick()
                            .is_some_and(|trick| trick.led_suit().is_none());
                        let update = session.act(seat, action).unwrap();
                        if let (true, Action::Play(card) | Action::Lead { card, .. }) =
                            (leading, action)
                        {
                            led[card.suit()] = true;
                        }
                        if let Action::Lead { card, .. } = action {
                            signalled[seat][card.suit()] = true;
                        }
                        assert_card_event(seat, action, &update);
                        if let Action::Declare(declaration) = action {
                            assert_declared(&session, config, seat, declaration, &update);
                        }
                        update
                    }
                }
                Status::Over(Outcome {
                    winner,
                    score,
                    forfeited,
                }) => {
                    match forfeited {
                        Some(seat) => assert_eq!(winner, seat.team().opponent()),
                        None => {
                            assert!(score[winner] >= SCORE_TO_WIN);
                            assert!(score[winner] > score[winner.opponent()]);
                        }
                    }
                    return session;
                }
            };

            assert_eq!(update.step() + 1, session.steps());
            for (audience, event) in update.events().iter() {
                assert_eq!(
                    *audience == Audience::Everyone,
                    !matches!(event, Event::CardsReceived { .. }),
                    "{event:?} has the wrong audience"
                );
                match event {
                    Event::HandDealt { .. } => {
                        declared = PerTeam::default();
                        hand_actions = 0;
                        signalled = PerSeat::default();
                        led = PerSuit::default();
                    }
                    Event::Declared { seat, points, .. } => declared[seat.team()] += points,
                    Event::HandScored { points, .. } => assert_hand_points(*points, declared),
                    _ => {}
                }
            }
            for seat in Seat::ALL {
                for event in update.for_seat(seat) {
                    trackers[seat].apply(Some(seat), event);
                }
                trackers[seat].assert_matches(&session.observe(seat));
            }
            for event in update.public_only() {
                spectator.apply(None, event);
            }
            spectator.assert_matches(&session.spectate());
            if let Some(hand) = session.game().phase.hand() {
                assert_cards_conserved(hand);
            }
        }
    }

    proptest! {
        // Checking every card on every turn is slow: fewer games suffice.
        #![proptest_config(ProptestConfig::with_cases(32))]

        /// Whole random games follow the rules under any declaration rules:
        /// exactly the legal actions are accepted, declarations are offered,
        /// announced and scored as the rules say, every seat's view matches
        /// what its events told it, and the log replays into the same game.
        #[test]
        fn random_games_follow_the_rules_and_replay(config in config_strategy(), deals: u64, choices: u64) {
            let session = play_out(config, deals, choices);
            let replay = Session::<Tressette>::replay(config, session.log().to_vec()).unwrap();
            prop_assert_eq!(replay, session);
        }
    }
}
