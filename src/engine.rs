//! The game engine: the [`Game`] trait every game implements, and the
//! [`Session`] that drives a game and records its history.
//!
//! A game is a state machine. At any point its [`Status`] says what it's
//! waiting for: a chance outcome ([`Status::Chance`], e.g. a deal), actions
//! from some seats ([`Status::ToAct`]), or nothing because it's over
//! ([`Status::Over`]).
//!
//! Games never check who may act: [`Session`] does, and hands the game a
//! [`Turn`] that only it can create. Chance outcomes are ordinary values:
//! the host either has the game sample one from a random number generator
//! ([`Session::resolve_chance`]), or supplies a specific one
//! ([`Session::resolve_chance_with`]), e.g. to replay a deal dealt with real
//! cards. Either way, the outcome is recorded, so a game is fully determined
//! by its [config](Game::Config) and its [`Step`]s, which is what makes
//! [`Session::replay`] possible.
//!
//! # Hidden information
//!
//! The engine is built for an authoritative server: the [`Session`] holds the
//! full state and never exposes it. Players and spectators learn about the
//! game in two ways:
//! - [`Session::observe`] and [`Session::spectate`] return everything a seat,
//!   or anyone watching, is allowed to know, e.g. for someone joining or
//!   reconnecting. From an [`Observation`] a player can also tell who the game
//!   is waiting on and which actions they may take.
//! - Every step returns an [`Update`]: a public [`Notice`] of what kind of
//!   step it was, so that everyone can follow the game's progress even when
//!   its content is private, and the game's [`Events`], each meant for
//!   everyone or for a single seat. [`Update::visible_to`] keeps what one
//!   seat, or the spectators, may receive.
//!
//! A seat that reconnects gets the current state from [`Session::observe`],
//! and the updates it missed from [`Session::updates_for`], starting from the
//! first step it didn't receive.
//!
//! This is the factored-observation model of Kovařík et al., "Rethinking
//! formal models of partially observable multiagent decision making" (2022):
//! public observations, plus private ones for each seat.
//!
//! # Leaving
//!
//! Whoever hosts the session decides when a seat is out of the game, e.g.
//! because it left, resigned or took too long to act, and tells the game with
//! [`Session::forfeit`]. What forfeiting means is up to the game, e.g. its
//! team losing.

use std::fmt::Debug;

use rand::Rng;

/// A turn-based game, implemented as a state machine.
///
/// Implementations only encode the rules: [`Session`] takes care of who may
/// act and of the history.
pub trait Game: Sized {
    /// The settings a game is created with, e.g. a board size. `()` for games
    /// that have none.
    type Config: Clone + Eq + Debug;
    /// A seat at the table.
    type Seat: Copy + Eq + Debug;
    /// Something a seat can do.
    type Action: Clone + Eq + Debug;
    /// The outcome of a chance step, e.g. a whole deal or a die roll. Games
    /// without randomness use [`Infallible`](std::convert::Infallible),
    /// which has no values: the compiler then knows chance never happens.
    type Chance: Clone + Eq + Debug;
    /// Something that happened as a result of a step.
    type Event: Clone + Debug;
    /// What a seat, or a spectator, is allowed to know about the game.
    type Observation: Observation<Seat = Self::Seat, Action = Self::Action, Outcome = Self::Outcome>
        + Clone
        + Debug;
    /// The result of a finished game.
    type Outcome: Clone + Debug;
    /// Why an action or a chance outcome breaks the rules.
    type Error: std::error::Error;

    /// Returns every seat at the table, in a fixed order. They stay the same
    /// for the whole game.
    fn seats(&self) -> &[Self::Seat];

    /// Creates a game, before any chance outcome or action.
    fn new(config: &Self::Config) -> Self;

    /// Returns what the game is waiting for.
    fn status(&self) -> Status<Self::Seat, Self::Outcome>;

    /// Applies `action` for the seat that holds `turn`.
    ///
    /// # Errors
    ///
    /// Returns an error if `action` breaks the rules, i.e. if it's not among
    /// the [`legal_actions`](Observation::legal_actions) of the acting seat's
    /// observation. The game must be left unchanged in that case.
    fn act(
        &mut self,
        turn: Turn<Self::Seat>,
        action: Self::Action,
    ) -> Result<Events<Self::Seat, Self::Event>, Self::Error>;

    /// Shuffles the deck, rolls the dice, or whatever chance step the game is
    /// waiting for, using `rng`, and returns the result without applying it.
    ///
    /// Returns `None` if the game isn't waiting for a chance step, which in
    /// games without chance is always.
    fn sample_chance(&self, rng: &mut impl Rng) -> Option<Self::Chance>;

    /// Applies `outcome` to the chance step the game is waiting for.
    ///
    /// # Errors
    ///
    /// Returns an error if `outcome` isn't a possible outcome of this chance
    /// step, e.g. a deal that doesn't use the whole deck. The game must be
    /// left unchanged in that case.
    fn apply_chance(
        &mut self,
        outcome: Self::Chance,
    ) -> Result<Events<Self::Seat, Self::Event>, Self::Error>;

    /// Applies the consequences of the seat that holds `turn` leaving the
    /// game, e.g. its team losing.
    ///
    /// The seat may forfeit at any point, even while the game waits for a
    /// chance outcome or on other seats. A game that goes on afterwards must
    /// not wait on the seat again: [`Session`] rejects its actions from then
    /// on, so the game would be stuck.
    fn forfeit(&mut self, turn: Turn<Self::Seat>) -> Events<Self::Seat, Self::Event>;

    /// Returns what `viewer` is allowed to know about the game.
    fn observe(&self, viewer: Viewer<Self::Seat>) -> Self::Observation;
}

/// A [`Step`] of the game `G`.
pub type StepOf<G> = Step<<G as Game>::Seat, <G as Game>::Action, <G as Game>::Chance>;

/// What a seat, or a spectator, is allowed to know about a game.
///
/// Whatever a seat may do only ever depends on what it knows, so an
/// observation is enough to tell which actions are legal.
pub trait Observation {
    /// A seat at the table.
    type Seat;
    /// Something a seat can do.
    type Action;
    /// The result of a finished game.
    type Outcome;
    /// The actions a seat may take, in whatever form suits the game: a list,
    /// or a compact description that's walked lazily, e.g. a range of bets.
    type Legal: IntoIterator<Item = Self::Action>;

    /// Returns what the game is waiting for.
    fn status(&self) -> Status<Self::Seat, Self::Outcome>;

    /// Returns the actions the observing seat may take. Empty when the game
    /// isn't waiting on it, and always for a spectator.
    fn legal_actions(&self) -> Self::Legal;
}

/// What a game is waiting for.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Status<S, O> {
    /// Waiting for a chance outcome, e.g. a deal. See
    /// [`Session::resolve_chance`].
    Chance,
    /// Waiting for these seats to act, in any order. See [`Session::act`].
    ToAct(Waiting<S>),
    /// The game is over.
    Over(O),
}

impl<S, O> Status<S, O> {
    /// Returns the seats the game is waiting on: empty unless it's
    /// [`ToAct`](Self::ToAct).
    pub fn to_act(&self) -> &[S] {
        match self {
            Self::ToAct(seats) => seats.as_slice(),
            Self::Chance | Self::Over(_) => &[],
        }
    }
}

/// The seats a game is waiting on: at least one, in a fixed order and
/// without duplicates.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Waiting<S> {
    seats: Seats<S>,
}

/// The seats of a [`Waiting`]. Most games wait on one seat at a time, which
/// is kept without allocating. `Several` always holds two seats or more, so
/// that equal sets of seats are stored the same way.
#[derive(Clone, PartialEq, Eq, Hash)]
enum Seats<S> {
    One(S),
    Several(Vec<S>),
}

impl<S: Eq> Waiting<S> {
    /// Returns a single seat.
    pub fn one(seat: S) -> Self {
        Self {
            seats: Seats::One(seat),
        }
    }

    /// Returns `seats`, in order, keeping the first of any duplicates, or
    /// `None` if there are none: a game always waits on someone.
    ///
    /// ```
    /// use shuftlib::engine::Waiting;
    ///
    /// assert_eq!(Waiting::new([2, 1, 2]).unwrap().as_slice(), [2, 1]);
    /// assert_eq!(Waiting::<u8>::new([]), None);
    /// ```
    pub fn new(seats: impl IntoIterator<Item = S>) -> Option<Self> {
        let mut unique = Vec::new();
        for seat in seats {
            if !unique.contains(&seat) {
                unique.push(seat);
            }
        }
        let seats = match unique.len() {
            0 => return None,
            1 => Seats::One(unique.pop()?),
            _ => Seats::Several(unique),
        };
        Some(Self { seats })
    }
}

impl<S: Debug> Debug for Waiting<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.as_slice()).finish()
    }
}

impl<S> Waiting<S> {
    /// Returns the seats, in order.
    pub fn as_slice(&self) -> &[S] {
        match &self.seats {
            Seats::One(seat) => std::slice::from_ref(seat),
            Seats::Several(seats) => seats,
        }
    }
}

/// Behaves like a slice of the seats: `waiting[0]`, `waiting.contains(..)`,
/// `waiting.iter()` and so on.
impl<S> std::ops::Deref for Waiting<S> {
    type Target = [S];

    fn deref(&self) -> &[S] {
        self.as_slice()
    }
}

impl<'a, S> IntoIterator for &'a Waiting<S> {
    type Item = &'a S;
    type IntoIter = std::slice::Iter<'a, S>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Who is looking at a game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Viewer<S> {
    /// A seat at the table, which sees its own private information.
    Seat(S),
    /// Anyone watching, who only sees what's public.
    Spectator,
}

impl<S: Copy> Viewer<S> {
    /// Returns the seat, or `None` for a spectator.
    pub fn seat(&self) -> Option<S> {
        match self {
            Self::Seat(seat) => Some(*seat),
            Self::Spectator => None,
        }
    }
}

/// Who may receive an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Audience<S> {
    /// Every seat, and anyone watching.
    Everyone,
    /// This seat only.
    Only(S),
}

impl<S: Eq> Audience<S> {
    /// Returns whether `viewer` may receive the event: everyone receives
    /// public events, and only its seat a private one.
    pub fn reaches(&self, viewer: &Viewer<S>) -> bool {
        match (self, viewer) {
            (Self::Everyone, _) => true,
            (Self::Only(only), Viewer::Seat(seat)) => only == seat,
            (Self::Only(_), Viewer::Spectator) => false,
        }
    }
}

/// The events a game produced for one step, in order, each with its
/// [`Audience`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Events<S, E> {
    events: Vec<(Audience<S>, E)>,
}

impl<S, E> Events<S, E> {
    /// Returns an empty list of events.
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// Adds an event everyone may receive.
    pub fn public(&mut self, event: E) {
        self.events.push((Audience::Everyone, event));
    }

    /// Adds an event only `seat` may receive.
    pub fn private(&mut self, seat: S, event: E) {
        self.events.push((Audience::Only(seat), event));
    }

    /// Returns every event with its audience, in order.
    pub fn iter(&self) -> impl Iterator<Item = (&Audience<S>, &E)> {
        self.events
            .iter()
            .map(|(audience, event)| (audience, event))
    }

    /// Returns the number of events.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Returns whether there are no events.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

impl<S: Eq, E> Events<S, E> {
    /// Returns the events `seat` may receive, in order.
    pub fn for_seat(&self, seat: S) -> impl Iterator<Item = &E> {
        self.reaching(Viewer::Seat(seat))
    }

    /// Returns the events everyone may receive, in order. This is what
    /// spectators get.
    pub fn public_only(&self) -> impl Iterator<Item = &E> {
        self.reaching(Viewer::Spectator)
    }

    /// Returns the events `viewer` may receive, in order.
    fn reaching(&self, viewer: Viewer<S>) -> impl Iterator<Item = &E> {
        self.entries_reaching(viewer).map(|(_, event)| event)
    }

    /// Returns the events `viewer` may receive with their audiences, in
    /// order. Every way of reading events for a viewer goes through here.
    fn entries_reaching(&self, viewer: Viewer<S>) -> impl Iterator<Item = &(Audience<S>, E)> {
        self.events
            .iter()
            .filter(move |(audience, _)| audience.reaches(&viewer))
    }
}

impl<S: Clone + Eq, E: Clone> Events<S, E> {
    /// Returns only the events `viewer` may receive, in order: for a seat,
    /// the public ones and its own; for a spectator, the public ones.
    pub fn visible_to(&self, viewer: Viewer<S>) -> Self {
        Self {
            events: self.entries_reaching(viewer).cloned().collect(),
        }
    }
}

impl<S, E> Default for Events<S, E> {
    fn default() -> Self {
        Self::new()
    }
}

/// What kind of step was taken. Every [`Update`] carries one, and it's
/// public: everyone can follow the game's progress, even when what happened
/// is private.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Notice<S> {
    /// A chance outcome was applied.
    Chance,
    /// `seat` acted.
    Acted {
        /// The seat that acted.
        seat: S,
    },
    /// `seat` forfeited.
    Forfeit {
        /// The seat that forfeited.
        seat: S,
    },
}

/// What one step did: its position in the game, its public [`Notice`], and
/// the game's [`Events`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Update<S, E> {
    step: usize,
    notice: Notice<S>,
    events: Events<S, E>,
}

impl<S, E> Update<S, E> {
    /// Returns the step's position in the game's history, from 0.
    pub fn step(&self) -> usize {
        self.step
    }

    /// Returns what kind of step it was.
    pub fn notice(&self) -> &Notice<S> {
        &self.notice
    }

    /// Returns the game's events, with their audiences.
    pub fn events(&self) -> &Events<S, E> {
        &self.events
    }
}

impl<S: Eq, E> Update<S, E> {
    /// Returns the game's events `seat` may receive, in order.
    pub fn for_seat(&self, seat: S) -> impl Iterator<Item = &E> {
        self.events.for_seat(seat)
    }

    /// Returns the game's events everyone may receive, in order.
    pub fn public_only(&self) -> impl Iterator<Item = &E> {
        self.events.public_only()
    }
}

impl<S: Clone + Eq, E: Clone> Update<S, E> {
    /// Returns this update with only the events `viewer` may receive: what
    /// to send to that seat, or to spectators.
    pub fn visible_to(&self, viewer: Viewer<S>) -> Self {
        Self {
            step: self.step,
            notice: self.notice.clone(),
            events: self.events.visible_to(viewer),
        }
    }
}

/// An [`Update`] of the game `G`.
pub type UpdateOf<G> = Update<<G as Game>::Seat, <G as Game>::Event>;

/// The seat taking a step, handed to [`Game::act`] and [`Game::forfeit`]
/// by [`Session`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Turn<S> {
    seat: S,
}

impl<S: Copy> Turn<S> {
    /// Returns the seat taking the step.
    pub fn seat(&self) -> S {
        self.seat
    }
}

/// One step of a game's history.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Step<S, A, C> {
    /// A chance outcome was applied.
    Chance(C),
    /// `seat` took `action`.
    Act {
        /// The seat that acted.
        seat: S,
        /// The action it took.
        action: A,
    },
    /// `seat` forfeited.
    Forfeit {
        /// The seat that forfeited.
        seat: S,
    },
}

/// Why [`Session::act`] rejected an action.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ActError<S: Debug, E> {
    /// The game is waiting for a chance outcome, not for an action.
    #[error("the game is waiting for a chance outcome, not for an action")]
    AwaitingChance,
    /// The game is over.
    #[error("the game is over")]
    GameOver,
    /// The game isn't waiting on the seat.
    #[error("the game isn't waiting on {seat:?}")]
    NotYourTurn {
        /// The seat that tried to act.
        seat: S,
    },
    /// The seat forfeited, in a game that goes on without it.
    #[error("{seat:?} forfeited")]
    Forfeited {
        /// The seat that tried to act.
        seat: S,
    },
    /// The action breaks the rules of the game.
    #[error(transparent)]
    Illegal(E),
}

/// What [`Session::act`] returns: the step's update, or why the action was
/// rejected.
pub type ActResult<G> = Result<UpdateOf<G>, ActError<<G as Game>::Seat, <G as Game>::Error>>;

/// Why [`Session::resolve_chance`] or [`Session::resolve_chance_with`]
/// failed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ChanceError<E> {
    /// The game isn't waiting for a chance outcome.
    #[error("the game is not waiting for a chance outcome")]
    NotAwaitingChance,
    /// The game is waiting for a chance outcome, but sampled none: its
    /// [`sample_chance`](Game::sample_chance) disagrees with its
    /// [`status`](Game::status).
    #[error("the game is waiting for a chance outcome, but sampled none")]
    NothingSampled,
    /// The outcome isn't possible at this chance step.
    #[error(transparent)]
    Illegal(E),
}

/// What [`Session::resolve_chance`] and [`Session::resolve_chance_with`]
/// return: the step's update, or why it failed.
pub type ChanceResult<G> = Result<UpdateOf<G>, ChanceError<<G as Game>::Error>>;

/// Why [`Session::forfeit`] failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ForfeitError<S: Debug> {
    /// The game is already over.
    #[error("the game is already over")]
    GameOver,
    /// The seat isn't one of the game's [seats](Game::seats).
    #[error("{seat:?} is not a seat of the game")]
    NotSeated {
        /// The seat that tried to forfeit.
        seat: S,
    },
    /// The seat already forfeited, in a game that goes on without it.
    #[error("{seat:?} already forfeited")]
    AlreadyForfeited {
        /// The seat that tried to forfeit again.
        seat: S,
    },
}

/// Why [`Session::replay`] failed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ReplayError<S: Debug, E> {
    /// A chance step was rejected.
    #[error("step {step}: {error}")]
    Chance {
        /// The position of the step in the replayed history.
        step: usize,
        /// Why the chance outcome was rejected.
        error: ChanceError<E>,
    },
    /// An action step was rejected.
    #[error("step {step}: {error}")]
    Act {
        /// The position of the step in the replayed history.
        step: usize,
        /// Why the action was rejected.
        error: ActError<S, E>,
    },
    /// A forfeit step was rejected.
    #[error("step {step}: {error}")]
    Forfeit {
        /// The position of the step in the replayed history.
        step: usize,
        /// Why the forfeit was rejected.
        error: ForfeitError<S>,
    },
}

/// A failed [`Session::replay`] of the game `G`.
pub type ReplayErrorOf<G> = ReplayError<<G as Game>::Seat, <G as Game>::Error>;

/// A game being played: the game itself, the config it was created with, and
/// the history of every step.
///
/// A session holds the full state of the game, hidden information included,
/// and never exposes it: players only get [`observe`](Self::observe) and the
/// [`Update`]s meant for them. Its [`log`](Self::log) records every chance
/// outcome, deals included, so it must be kept from players until the game
/// is over.
#[derive(Clone, PartialEq, Eq)]
pub struct Session<G: Game> {
    game: G,
    config: G::Config,
    log: Vec<StepOf<G>>,
    /// The seats that forfeited, in order.
    forfeited: Vec<G::Seat>,
}

/// Shows the config, what the game is waiting for and how many steps were
/// applied, but neither the game nor its log: both hold hidden information.
impl<G: Game> Debug for Session<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("config", &self.config)
            .field("status", &self.status())
            .field("steps", &self.log.len())
            .finish_non_exhaustive()
    }
}

impl<G: Game> Session<G> {
    /// Starts a new game with `config`.
    pub fn new(config: G::Config) -> Self {
        Self {
            game: G::new(&config),
            config,
            log: Vec::new(),
            forfeited: Vec::new(),
        }
    }

    /// Rebuilds a session by applying `steps` to a new game created with
    /// `config`.
    ///
    /// # Errors
    ///
    /// Returns an error if a step can't be applied.
    pub fn replay(
        config: G::Config,
        steps: impl IntoIterator<Item = StepOf<G>>,
    ) -> Result<Self, ReplayErrorOf<G>> {
        let mut session = Self::new(config);
        for (step, entry) in steps.into_iter().enumerate() {
            session.apply(step, entry)?;
        }
        Ok(session)
    }

    /// Applies one step of a replayed history, at position `step`.
    fn apply(&mut self, step: usize, entry: StepOf<G>) -> Result<UpdateOf<G>, ReplayErrorOf<G>> {
        match entry {
            Step::Chance(outcome) => self
                .resolve_chance_with(outcome)
                .map_err(|error| ReplayError::Chance { step, error }),
            Step::Act { seat, action } => self
                .act(seat, action)
                .map_err(|error| ReplayError::Act { step, error }),
            Step::Forfeit { seat } => self
                .forfeit(seat)
                .map_err(|error| ReplayError::Forfeit { step, error }),
        }
    }

    /// Returns what the game is waiting for.
    pub fn status(&self) -> Status<G::Seat, G::Outcome> {
        self.game.status()
    }

    /// Applies `action` on behalf of `seat`.
    ///
    /// # Errors
    ///
    /// Returns an error, leaving the game unchanged, if the game isn't
    /// waiting for actions, if `seat` forfeited, if the game isn't waiting
    /// on `seat` or if `action` breaks the rules.
    pub fn act(&mut self, seat: G::Seat, action: G::Action) -> ActResult<G> {
        match self.game.status() {
            Status::Chance => return Err(ActError::AwaitingChance),
            Status::Over(_) => return Err(ActError::GameOver),
            Status::ToAct(_) if self.forfeited.contains(&seat) => {
                return Err(ActError::Forfeited { seat });
            }
            Status::ToAct(waiting) if !waiting.contains(&seat) => {
                return Err(ActError::NotYourTurn { seat });
            }
            Status::ToAct(_) => {}
        }
        let events = self
            .game
            .act(Turn { seat }, action.clone())
            .map_err(ActError::Illegal)?;
        Ok(self.record(Step::Act { seat, action }, Notice::Acted { seat }, events))
    }

    /// Draws the chance outcome the game is waiting for from `rng`, and
    /// applies it.
    ///
    /// # Errors
    ///
    /// Returns an error, leaving the game unchanged, if the game isn't
    /// waiting for a chance outcome, or if it is but its
    /// [`sample_chance`](Game::sample_chance) draws none.
    pub fn resolve_chance(&mut self, rng: &mut impl Rng) -> ChanceResult<G> {
        if !matches!(self.game.status(), Status::Chance) {
            return Err(ChanceError::NotAwaitingChance);
        }
        let outcome = self
            .game
            .sample_chance(rng)
            .ok_or(ChanceError::NothingSampled)?;
        self.apply_chance(outcome)
    }

    /// Applies `outcome` to the chance step the game is waiting for, e.g. a
    /// deal dealt with real cards, or a position to analyze.
    ///
    /// # Errors
    ///
    /// Returns an error, leaving the game unchanged, if the game isn't
    /// waiting for a chance outcome or if `outcome` isn't possible.
    pub fn resolve_chance_with(&mut self, outcome: G::Chance) -> ChanceResult<G> {
        if !matches!(self.game.status(), Status::Chance) {
            return Err(ChanceError::NotAwaitingChance);
        }
        self.apply_chance(outcome)
    }

    /// Applies `outcome` to the chance step the game is known to be waiting
    /// for.
    fn apply_chance(&mut self, outcome: G::Chance) -> ChanceResult<G> {
        let events = self
            .game
            .apply_chance(outcome.clone())
            .map_err(ChanceError::Illegal)?;
        Ok(self.record(Step::Chance(outcome), Notice::Chance, events))
    }

    /// Takes `seat` out of the game, for whatever reason the host has, e.g.
    /// it left, resigned or took too long to act. What that means is up to
    /// the game.
    ///
    /// # Errors
    ///
    /// Returns an error, leaving the game unchanged, if the game is already
    /// over, if `seat` isn't one of its [seats](Self::seats), or if it
    /// already forfeited.
    pub fn forfeit(&mut self, seat: G::Seat) -> Result<UpdateOf<G>, ForfeitError<G::Seat>> {
        if matches!(self.game.status(), Status::Over(_)) {
            return Err(ForfeitError::GameOver);
        }
        if !self.seats().contains(&seat) {
            return Err(ForfeitError::NotSeated { seat });
        }
        if self.forfeited.contains(&seat) {
            return Err(ForfeitError::AlreadyForfeited { seat });
        }
        let events = self.game.forfeit(Turn { seat });
        self.forfeited.push(seat);
        Ok(self.record(Step::Forfeit { seat }, Notice::Forfeit { seat }, events))
    }

    fn record(
        &mut self,
        step: StepOf<G>,
        notice: Notice<G::Seat>,
        events: Events<G::Seat, G::Event>,
    ) -> UpdateOf<G> {
        let index = self.log.len();
        self.log.push(step);
        Update {
            step: index,
            notice,
            events,
        }
    }

    /// Returns what `seat` is allowed to know about the game.
    pub fn observe(&self, seat: G::Seat) -> G::Observation {
        self.game.observe(Viewer::Seat(seat))
    }

    /// Returns what anyone watching is allowed to know about the game.
    pub fn spectate(&self) -> G::Observation {
        self.game.observe(Viewer::Spectator)
    }

    /// Returns every seat of the game, in a fixed order.
    pub fn seats(&self) -> &[G::Seat] {
        self.game.seats()
    }

    /// Returns the config the game was created with.
    pub fn config(&self) -> &G::Config {
        &self.config
    }

    /// Returns every step applied so far, in order. It reveals every chance
    /// outcome, see the [type documentation](Self).
    pub fn log(&self) -> &[StepOf<G>] {
        &self.log
    }

    /// Returns the seats that forfeited, in the order they did. They may no
    /// longer act, nor forfeit again.
    pub fn forfeited(&self) -> &[G::Seat] {
        &self.forfeited
    }

    /// Returns how many steps were applied: the game's public clock.
    pub fn steps(&self) -> usize {
        self.log.len()
    }

    /// Returns the updates of every step from `since` on, each as
    /// [`visible_to`](Update::visible_to) `viewer` would give it: exactly
    /// what that seat, or a spectator, was sent. A seat that reconnects asks
    /// from the first step it missed, and one starting over asks from 0 to
    /// rebuild its whole history. Empty if `since` is past the last step.
    ///
    /// The updates are rebuilt by [replaying](Self::replay) the log, so this
    /// takes as long as the whole game's rules do.
    ///
    /// # Errors
    ///
    /// Returns an error if the log can't be replayed, which only happens if
    /// the game's rules aren't deterministic.
    pub fn updates_for(
        &self,
        viewer: Viewer<G::Seat>,
        since: usize,
    ) -> Result<Vec<UpdateOf<G>>, ReplayErrorOf<G>> {
        // A seat that's up to date needs nothing, and no replay.
        if since >= self.log.len() {
            return Ok(Vec::new());
        }
        let mut replay = Self::new(self.config.clone());
        let mut updates = Vec::new();
        for (step, entry) in self.log.iter().cloned().enumerate() {
            let update = replay.apply(step, entry)?;
            if step >= since {
                updates.push(update.visible_to(viewer));
            }
        }
        Ok(updates)
    }

    /// Starts a session from an arbitrary game state, e.g. a specific
    /// position. Its log can't be replayed.
    #[cfg(test)]
    pub(crate) fn from_game(config: G::Config, game: G) -> Self {
        Self {
            game,
            config,
            log: Vec::new(),
            forfeited: Vec::new(),
        }
    }

    /// Returns the full state of the game, to check its invariants.
    #[cfg(test)]
    pub(crate) fn game(&self) -> &G {
        &self.game
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use proptest::{collection::vec, option, prelude::*};
    use rand::{RngExt, SeedableRng, rngs::StdRng, seq::IndexedRandom};

    use super::*;

    /// Two seats take turns guessing a secret number below the config,
    /// drawn at random. The first seat is told whether the secret is even;
    /// the first correct guess wins, and so does the opponent of a seat that
    /// forfeits.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(crate) struct Guessing {
        below: u8,
        secret: Option<u8>,
        to_act: Seat,
        winner: Option<Seat>,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) enum Seat {
        First,
        Second,
    }

    impl Seat {
        fn other(self) -> Self {
            match self {
                Self::First => Self::Second,
                Self::Second => Self::First,
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) enum Action {
        Guess(u8),
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) enum Event {
        SecretIsEven(bool),
        Guessed { seat: Seat, correct: bool },
        Lost { seat: Seat },
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(crate) struct View {
        viewer: Viewer<Seat>,
        below: u8,
        status: Status<Seat, Seat>,
    }

    impl Observation for View {
        type Seat = Seat;
        type Action = Action;
        type Outcome = Seat;
        type Legal = std::iter::Map<Range<u8>, fn(u8) -> Action>;

        fn status(&self) -> Status<Seat, Seat> {
            self.status.clone()
        }

        fn legal_actions(&self) -> Self::Legal {
            let guesses = match self.viewer.seat() {
                Some(seat) if self.status.to_act().contains(&seat) => 0..self.below,
                _ => 0..0,
            };
            guesses.map(Action::Guess as fn(u8) -> Action)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
    pub(crate) enum Error {
        #[error("numbers must be below the config")]
        OutOfRange,
    }

    impl Game for Guessing {
        type Config = u8;
        type Seat = Seat;
        type Action = Action;
        type Chance = u8;
        type Event = Event;
        type Observation = View;
        type Outcome = Seat;
        type Error = Error;

        fn seats(&self) -> &[Seat] {
            &[Seat::First, Seat::Second]
        }

        fn new(below: &u8) -> Self {
            Self {
                below: *below,
                secret: None,
                to_act: Seat::First,
                winner: None,
            }
        }

        fn status(&self) -> Status<Seat, Seat> {
            match (self.winner, self.secret) {
                (Some(winner), _) => Status::Over(winner),
                (None, None) => Status::Chance,
                (None, Some(_)) => Status::ToAct(Waiting::one(self.to_act)),
            }
        }

        fn act(&mut self, turn: Turn<Seat>, action: Action) -> Result<Events<Seat, Event>, Error> {
            let seat = turn.seat();
            let Action::Guess(guess) = action;
            if guess >= self.below {
                return Err(Error::OutOfRange);
            }
            let correct = self.secret == Some(guess);
            if correct {
                self.winner = Some(seat);
            }
            self.to_act = seat.other();
            let mut events = Events::new();
            events.public(Event::Guessed { seat, correct });
            Ok(events)
        }

        fn sample_chance(&self, rng: &mut impl Rng) -> Option<u8> {
            (self.status() == Status::Chance).then(|| rng.random_range(0..self.below))
        }

        fn apply_chance(&mut self, secret: u8) -> Result<Events<Seat, Event>, Error> {
            if secret >= self.below {
                return Err(Error::OutOfRange);
            }
            self.secret = Some(secret);
            let mut events = Events::new();
            events.private(Seat::First, Event::SecretIsEven(secret.is_multiple_of(2)));
            Ok(events)
        }

        fn forfeit(&mut self, turn: Turn<Seat>) -> Events<Seat, Event> {
            let seat = turn.seat();
            self.winner = Some(seat.other());
            let mut events = Events::new();
            events.public(Event::Lost { seat });
            events
        }

        fn observe(&self, viewer: Viewer<Seat>) -> View {
            View {
                viewer,
                below: self.below,
                status: self.status(),
            }
        }
    }

    /// Plays like [`Guessing`], with two twists depending on its config.
    /// With 0, it breaks the promises of the engine: it has no seats, and
    /// samples no secret while waiting for one. With 1, a seat that forfeits
    /// is out but the game goes on.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(crate) struct Faulty(Guessing);

    impl Game for Faulty {
        type Config = u8;
        type Seat = Seat;
        type Action = Action;
        type Chance = u8;
        type Event = Event;
        type Observation = View;
        type Outcome = Seat;
        type Error = Error;

        fn seats(&self) -> &[Seat] {
            if self.0.below == 0 {
                return &[];
            }
            self.0.seats()
        }

        fn new(below: &u8) -> Self {
            Self(Guessing::new(below))
        }

        fn status(&self) -> Status<Seat, Seat> {
            self.0.status()
        }

        fn act(&mut self, turn: Turn<Seat>, action: Action) -> Result<Events<Seat, Event>, Error> {
            self.0.act(turn, action)
        }

        fn sample_chance(&self, rng: &mut impl Rng) -> Option<u8> {
            if self.0.below == 0 {
                return None;
            }
            self.0.sample_chance(rng)
        }

        fn apply_chance(&mut self, secret: u8) -> Result<Events<Seat, Event>, Error> {
            self.0.apply_chance(secret)
        }

        fn forfeit(&mut self, turn: Turn<Seat>) -> Events<Seat, Event> {
            if self.0.below == 1 {
                let mut events = Events::new();
                events.public(Event::Lost { seat: turn.seat() });
                return events;
            }
            self.0.forfeit(turn)
        }

        fn observe(&self, viewer: Viewer<Seat>) -> View {
            self.0.observe(viewer)
        }
    }

    fn play_out(seed: u64) -> Session<Guessing> {
        let mut session = Session::<Guessing>::new(10);
        session
            .resolve_chance(&mut StdRng::seed_from_u64(seed))
            .unwrap();
        let mut guess = 0;
        while let Status::ToAct(seats) = session.status() {
            session
                .act(seats.as_slice()[0], Action::Guess(guess))
                .unwrap();
            guess += 1;
        }
        session
    }

    fn started(secret: u8) -> Session<Guessing> {
        let mut session = Session::<Guessing>::new(10);
        session.resolve_chance_with(secret).unwrap();
        session
    }

    #[test]
    fn acting_before_chance_is_rejected() {
        let mut session = Session::<Guessing>::new(10);
        assert_eq!(
            session.act(Seat::First, Action::Guess(0)),
            Err(ActError::AwaitingChance)
        );
        assert!(session.log().is_empty());
    }

    #[test]
    fn acting_out_of_turn_is_rejected() {
        let mut session = started(5);
        session.act(Seat::First, Action::Guess(0)).unwrap();
        let before = session.clone();
        assert_eq!(
            session.act(Seat::First, Action::Guess(1)),
            Err(ActError::NotYourTurn { seat: Seat::First })
        );
        assert_eq!(session, before);
    }

    #[test]
    fn illegal_actions_are_rejected_without_changing_the_game() {
        let mut session = started(5);
        let before = session.clone();
        assert_eq!(
            session.act(Seat::First, Action::Guess(10)),
            Err(ActError::Illegal(Error::OutOfRange))
        );
        assert_eq!(session, before);
    }

    #[test]
    fn chance_outcomes_can_be_forced_and_are_checked() {
        let mut session = Session::<Guessing>::new(10);
        let before = session.clone();
        assert_eq!(
            session.resolve_chance_with(10),
            Err(ChanceError::Illegal(Error::OutOfRange))
        );
        assert_eq!(session, before);

        let update = session.resolve_chance_with(4).unwrap();
        assert_eq!(update.notice(), &Notice::Chance);
        assert_eq!(
            update.for_seat(Seat::First).collect::<Vec<_>>(),
            [&Event::SecretIsEven(true)]
        );
        assert_eq!(session.log(), [Step::Chance(4)]);
        assert_eq!(
            session.resolve_chance_with(4),
            Err(ChanceError::NotAwaitingChance)
        );
        assert_eq!(
            session.resolve_chance(&mut StdRng::seed_from_u64(1)),
            Err(ChanceError::NotAwaitingChance)
        );
    }

    #[test]
    fn finished_games_reject_actions() {
        let mut session = play_out(7);
        assert!(matches!(session.status(), Status::Over(_)));
        assert_eq!(
            session.act(Seat::First, Action::Guess(0)),
            Err(ActError::GameOver)
        );
    }

    #[test]
    fn any_seat_may_forfeit_until_the_game_is_over() {
        // Even before the game waits on anyone.
        let mut session = Session::<Guessing>::new(10);
        session.forfeit(Seat::Second).unwrap();
        assert_eq!(session.status(), Status::Over(Seat::First));

        // Even a seat the game isn't waiting on.
        let mut session = started(5);
        let update = session.forfeit(Seat::Second).unwrap();
        assert_eq!(update.notice(), &Notice::Forfeit { seat: Seat::Second });
        assert_eq!(
            update.public_only().collect::<Vec<_>>(),
            [&Event::Lost { seat: Seat::Second }]
        );
        assert_eq!(session.status(), Status::Over(Seat::First));
        assert_eq!(
            session.log(),
            [Step::Chance(5), Step::Forfeit { seat: Seat::Second }]
        );

        let before = session.clone();
        assert_eq!(session.forfeit(Seat::First), Err(ForfeitError::GameOver));
        assert_eq!(session, before);
    }

    #[test]
    fn a_seat_forfeits_only_once() {
        // In this game, a forfeit doesn't end it.
        let mut session = Session::<Faulty>::new(1);
        session.forfeit(Seat::Second).unwrap();
        let before = session.clone();
        assert_eq!(
            session.forfeit(Seat::Second),
            Err(ForfeitError::AlreadyForfeited { seat: Seat::Second })
        );
        assert_eq!(session, before);
        session.forfeit(Seat::First).unwrap();
        assert_eq!(session.forfeited(), [Seat::Second, Seat::First]);
    }

    #[test]
    fn a_seat_that_forfeited_may_not_act() {
        // In this game, a forfeit doesn't end it, and the first seat still
        // gets its turn.
        let mut session = Session::<Faulty>::new(1);
        session.forfeit(Seat::First).unwrap();
        session.resolve_chance_with(0).unwrap();
        assert_eq!(session.status().to_act(), [Seat::First]);
        let before = session.clone();
        assert_eq!(
            session.act(Seat::First, Action::Guess(0)),
            Err(ActError::Forfeited { seat: Seat::First })
        );
        assert_eq!(session, before);
    }

    #[test]
    fn only_the_seats_of_the_game_may_forfeit() {
        let mut session = Session::<Faulty>::new(0);
        let before = session.clone();
        assert_eq!(
            session.forfeit(Seat::First),
            Err(ForfeitError::NotSeated { seat: Seat::First })
        );
        assert_eq!(session, before);
    }

    #[test]
    fn games_that_sample_nothing_while_awaiting_chance_are_reported() {
        let mut session = Session::<Faulty>::new(0);
        assert_eq!(session.status(), Status::Chance);
        assert_eq!(
            session.resolve_chance(&mut StdRng::seed_from_u64(1)),
            Err(ChanceError::NothingSampled)
        );
        assert!(session.log().is_empty());
    }

    #[test]
    fn every_update_carries_its_step_and_a_public_notice() {
        let mut session = Session::<Guessing>::new(10);
        let updates = [
            session.resolve_chance_with(3).unwrap(),
            session.act(Seat::First, Action::Guess(0)).unwrap(),
            session.forfeit(Seat::Second).unwrap(),
        ];
        assert_eq!(updates.each_ref().map(Update::step), [0, 1, 2]);
        assert_eq!(
            updates.each_ref().map(|update| *update.notice()),
            [
                Notice::Chance,
                Notice::Acted { seat: Seat::First },
                Notice::Forfeit { seat: Seat::Second },
            ]
        );
        assert_eq!(session.steps(), 3);
        // The second seat receives nothing from the chance step but its
        // notice.
        assert_eq!(updates[0].for_seat(Seat::Second).count(), 0);
    }

    /// Plays a game of [`Guessing`] for numbers `below`, with random guesses
    /// and the odd forfeit drawn from `seed`, until it's over. Returns it
    /// with every update it produced.
    fn random_game(below: u8, seed: u64) -> (Session<Guessing>, Vec<UpdateOf<Guessing>>) {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut session = Session::<Guessing>::new(below);
        let mut sent = Vec::new();
        loop {
            let update = match session.status() {
                Status::Chance => session.resolve_chance(&mut rng).unwrap(),
                Status::ToAct(seats) => {
                    if rng.random_ratio(1, 20) {
                        let seat = *session.seats().choose(&mut rng).unwrap();
                        session.forfeit(seat).unwrap()
                    } else {
                        let guess = Action::Guess(rng.random_range(0..below));
                        session.act(seats[0], guess).unwrap()
                    }
                }
                Status::Over(_) => return (session, sent),
            };
            sent.push(update);
        }
    }

    fn viewer() -> impl Strategy<Value = Viewer<Seat>> {
        prop_oneof![
            Just(Viewer::Seat(Seat::First)),
            Just(Viewer::Seat(Seat::Second)),
            Just(Viewer::Spectator),
        ]
    }

    proptest! {
        #[test]
        fn replay_reproduces_the_game_from_its_log_alone(below in 1u8..20, seed: u64) {
            let (session, _) = random_game(below, seed);
            let replay = Session::<Guessing>::replay(below, session.log().to_vec()).unwrap();
            prop_assert_eq!(replay, session);
        }

        /// Whatever step a seat or spectator reconnects from, even one past
        /// the end, it gets exactly the updates it was sent from then on.
        #[test]
        fn reconnecting_from_any_step_gives_what_was_sent(
            below in 1u8..20,
            seed: u64,
            viewer in viewer(),
            since in 0usize..40,
        ) {
            let (session, sent) = random_game(below, seed);
            // From the start, the whole history; from `since`, the rest.
            for since in [0, since] {
                let expected: Vec<_> = sent
                    .iter()
                    .skip(since)
                    .map(|update| update.visible_to(viewer))
                    .collect();
                prop_assert_eq!(session.updates_for(viewer, since).unwrap(), expected);
            }
        }

        #[test]
        fn waiting_keeps_each_seat_once_in_order_of_first_appearance(
            seats in vec(0u8..8, 0..20),
        ) {
            let waiting = Waiting::new(seats.clone());
            // Nobody to wait on isn't a wait, and a single seat is the same
            // wait however it's built.
            prop_assert_eq!(waiting.is_none(), seats.is_empty());
            if let Some(&only) = seats.first().filter(|&&first| seats.iter().all(|&seat| seat == first)) {
                prop_assert_eq!(waiting.clone(), Some(Waiting::one(only)));
            }
            let waiting = waiting.map_or(Vec::new(), |waiting| waiting.to_vec());
            let first_appearances: Vec<u8> = seats
                .iter()
                .enumerate()
                .filter(|&(index, seat)| seats.iter().position(|other| other == seat) == Some(index))
                .map(|(_, &seat)| seat)
                .collect();
            prop_assert_eq!(waiting.as_slice(), first_appearances);
        }

        /// A seat gets the public events and its own, a spectator only the
        /// public ones, each in its original place in the order.
        #[test]
        fn events_are_visible_to_their_audience(
            events in vec((option::of(0u8..4), any::<u16>()), 0..20),
            seat in 0u8..4,
        ) {
            let mut all = Events::new();
            for &(only, event) in &events {
                match only {
                    Some(only) => all.private(only, event),
                    None => all.public(event),
                }
            }
            let kept = |viewer: Viewer<u8>| -> Vec<(Audience<u8>, u16)> {
                all.visible_to(viewer)
                    .iter()
                    .map(|(audience, &event)| (*audience, event))
                    .collect()
            };
            let expected = |include_own: bool| -> Vec<(Audience<u8>, u16)> {
                events
                    .iter()
                    .filter(|(only, _)| only.is_none() || (include_own && *only == Some(seat)))
                    .map(|&(only, event)| (only.map_or(Audience::Everyone, Audience::Only), event))
                    .collect()
            };
            prop_assert_eq!(kept(Viewer::Seat(seat)), expected(true));
            prop_assert_eq!(kept(Viewer::Spectator), expected(false));
        }
    }

    #[test]
    fn replay_reports_the_failing_step() {
        assert_eq!(
            Session::<Guessing>::replay(10, [Step::Chance(1), Step::Chance(2)]).unwrap_err(),
            ReplayError::Chance {
                step: 1,
                error: ChanceError::NotAwaitingChance,
            }
        );
        assert_eq!(
            Session::<Guessing>::replay(
                10,
                [
                    Step::Forfeit { seat: Seat::First },
                    Step::Forfeit { seat: Seat::Second },
                ]
            )
            .unwrap_err(),
            ReplayError::Forfeit {
                step: 1,
                error: ForfeitError::GameOver,
            }
        );
    }

    #[test]
    fn debug_output_hides_the_game_and_the_log() {
        let session = started(5);
        assert_eq!(
            format!("{session:?}"),
            "Session { config: 10, status: ToAct([First]), steps: 1, .. }"
        );
    }
}
