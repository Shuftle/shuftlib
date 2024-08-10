use std::{fmt::Display, ops::Deref};

use anyhow::bail;

use super::cards::Card;

/// Many of the types contained in  this module are generic over certain
/// constants related to the game. This trait is the summary of these
/// constraints.
pub trait TrickTakingGame {
    /// Define the type of card that's going to be used in this game.
    type CardType: Card;
    /// Every game has a fixed number of players defined by the rules of the
    /// game or, anyway, before starting it.
    const PLAYERS: usize;
    /// Usually trick taking games have a fixed number of "turns" for each
    /// player. These "turns" are called tricks
    const TRICKS: usize;

    /// Every trick taking game has some logic to determine the winner (or
    /// taker) of the trick. The taker is generally determined by the cards that
    /// have been played and it can depend by the order in which the players
    /// played their cards.
    fn determine_taker(
        cards: &[Self::CardType; Self::PLAYERS],
        first_to_play: PlayerId<{ Self::PLAYERS }>,
    ) -> PlayerId<{ Self::PLAYERS }>;
}

/// Represents a player of a game. This type is generic over the type of the
/// card used for the specific game and over the number of players of such game.
#[derive(Clone, Default, Debug)]
pub struct Player<G>
where
    G: TrickTakingGame,
    [(); G::PLAYERS]:,
{
    /// The cards traditionally held in the hand by the player.
    hand: Vec<G::CardType>,
    /// The ID of this player. This is used to determine their turn to play.
    id: PlayerId<{ G::PLAYERS }>,
}

impl<G> Player<G>
where
    G: TrickTakingGame,
    [(); G::PLAYERS]:,
{
    /// Adds a card to the hand of the player.
    ///
    /// # Examples
    /// ```
    /// use shuftlib::common::hands::{Player, TrickTakingGame, PlayerId};
    /// use shuftlib::common::cards::{ItalianRank, Suit};
    /// use shuftlib::tressette::{TressetteRules, TressetteCard};
    ///
    /// let player_id = PlayerId::<{TressetteRules::PLAYERS}>::new(0).unwrap();
    /// let mut player = Player::<TressetteRules>::new(player_id);
    /// // Players have no cards when created.
    /// assert_eq!(player.hand().len(), 0);
    ///
    /// let card = TressetteCard::new(ItalianRank::Ace, Suit::Spades);
    /// player.give(card);
    /// assert_eq!(player.hand().len(), 1);
    /// ```
    pub fn give(&mut self, card: G::CardType) {
        self.hand.push(card);
    }

    /// Removes a card from the hand of the player.
    ///
    /// # Examples
    /// ```
    /// use shuftlib::common::hands::{Player, TrickTakingGame, PlayerId};
    /// use shuftlib::common::cards::{ItalianRank, Suit};
    /// use shuftlib::tressette::{TressetteRules, TressetteCard};
    ///
    /// let player_id = PlayerId::<{TressetteRules::PLAYERS}>::new(0).unwrap();
    /// let mut player = Player::<TressetteRules>::new(player_id);
    /// // Players have no cards when created.
    /// assert_eq!(player.hand().len(), 0);
    ///
    /// let card = TressetteCard::new(ItalianRank::Ace, Suit::Spades);
    /// player.give(card);
    /// assert_eq!(player.hand().len(), 1);
    ///
    /// player.remove(card);
    /// assert_eq!(player.hand().len(), 0);
    /// ```
    pub fn remove(&mut self, card: G::CardType) {
        self.hand.retain(|&c| c != card);
    }

    /// Getter for the cards held by this player.
    pub fn hand(&self) -> &[G::CardType] {
        &self.hand
    }

    /// Getter for the id of this player.
    pub fn id(&self) -> PlayerId<{ G::PLAYERS }> {
        self.id
    }

    /// Generates a new player from a `PlayerId`. Players are initialized with
    /// no cards.
    ///
    /// # Examples.
    ///
    /// ```
    /// use shuftlib::{common::hands::{Player, PlayerId, TrickTakingGame}, tressette::TressetteRules};
    ///
    /// let id = PlayerId::<{TressetteRules::PLAYERS}>::new(0).unwrap();
    /// let player = Player::<TressetteRules>::new(id);
    ///
    /// assert_eq!(*player.id(), 0);
    /// assert_eq!(player.hand().len(), 0);
    /// ````
    pub fn new(id: PlayerId<{ G::PLAYERS }>) -> Self {
        Self {
            id,
            hand: Vec::new(),
        }
    }
}

/// A player id can only be in the range 0..N, where N depends on the game being
/// played and it's the number of players playing that specific game.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct PlayerId<const PLAYERS: usize>(usize);

impl<const PLAYERS: usize> PlayerId<PLAYERS> {
    /// This method simply increments `self` by 1. Note that `PlayerId` can only
    /// be in the range 0..N, so incrementing `self` when the value is N-1, will
    /// reset its value to 0, since the purpose of this type is to determine the
    /// player's turn and the first person to play is not necessarily the person
    /// with ID = 0.
    ///
    /// # Examples
    /// ```
    /// #![feature(generic_const_exprs)]
    /// use shuftlib::common::hands::PlayerId;
    ///
    /// let mut player_id: PlayerId<4>= PlayerId::new(0).unwrap();
    /// player_id.inc();
    /// assert_eq!(player_id, PlayerId::<4>::new(1).unwrap());
    /// player_id.inc();
    /// player_id.inc();
    /// player_id.inc();
    /// assert_eq!(player_id, PlayerId::<4>::new(0).unwrap());
    /// ```
    pub fn inc(&mut self) {
        if self.0 < PLAYERS - 1 {
            self.0 += 1;
        } else {
            self.0 = 0;
        }
    }

    /// Creates a value of type `PlayerId`. Returns None if value is >= N,
    /// otherwise returns Some(PlayerId(value)).
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::common::hands::PlayerId;
    ///
    /// let id = PlayerId::<4>::new(0);
    /// assert!(id.is_some());
    ///
    /// let id = PlayerId::<4>::new(4);
    /// assert!(id.is_none());
    /// ```
    pub fn new(value: usize) -> Option<Self> {
        if value < PLAYERS {
            Some(PlayerId(value))
        } else {
            None
        }
    }
}

impl<const PLAYERS: usize> TryFrom<usize> for PlayerId<PLAYERS> {
    type Error = anyhow::Error;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if (0..PLAYERS).contains(&value) {
            Ok(PlayerId(value))
        } else {
            bail!(
                "Tried to convert {} into a PlayerId, but acceptable values are in range 0..PLAYERS",
                value
            )
        }
    }
}

impl<const PLAYERS: usize> Display for PlayerId<PLAYERS> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<const PLAYERS: usize> Deref for PlayerId<PLAYERS> {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A trick is a set containing the cards played and the player who won the
/// trick, represented as `PlayerId`.
#[derive(Debug, Copy, Clone)]
pub struct Trick<G>
where
    G: TrickTakingGame,
    [(); G::PLAYERS]:,
{
    cards: [G::CardType; G::PLAYERS],
    taker: PlayerId<{ G::PLAYERS }>,
}

impl<G> Display for Trick<G>
where
    G: TrickTakingGame,
    [(); G::PLAYERS]:,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.cards[0], self.cards[1], self.cards[2], self.cards[3], self.taker
        )
    }
}

impl<G> Trick<G>
where
    G: TrickTakingGame,
    [(); G::PLAYERS]:,
{
    /// Returns the card this trick has been won with.
    pub fn taken_with(&self) -> G::CardType {
        self.cards[self.taker.0]
    }

    /// Getter for the `PlayerId` of the player who won the trick.
    pub fn taker(&self) -> PlayerId<{ G::PLAYERS }> {
        self.taker
    }

    /// Getter for the cards played during this trick.
    pub fn cards(&self) -> &[G::CardType] {
        &self.cards
    }
}

/// A temporary state of a trick that's still not over: not all the players made
/// their move or a taker hasn't been determined yet.
#[derive(Clone, Copy, Debug)]
pub struct OngoingTrick<G>
where
    G: TrickTakingGame,
    [(); G::PLAYERS]:,
{
    cards: [Option<G::CardType>; G::PLAYERS],
    first_to_play: PlayerId<{ G::PLAYERS }>,
    next_to_play: PlayerId<{ G::PLAYERS }>,
    play_count: usize,
}

impl<G> Deref for OngoingTrick<G>
where
    G: TrickTakingGame,
    [(); G::PLAYERS]:,
{
    type Target = [Option<G::CardType>; G::PLAYERS];

    fn deref(&self) -> &Self::Target {
        &self.cards
    }
}

impl<G> OngoingTrick<G>
where
    G: TrickTakingGame,
    [(); G::PLAYERS]:,
{
    /// Adds the `Card` passed as parameter to the `OngoingTrick`.
    /// Checking the validity of the card played is a responsability of the
    /// caller.
    ///
    /// # Examples
    /// ```
    /// #![feature(generic_const_exprs)]
    /// use shuftlib::common::{hands::{OngoingTrick, PlayerId, TrickTakingGame}, cards::{Card, ItalianRank, Suit}};
    /// use shuftlib::tressette::{TressetteRules, TressetteCard};
    ///
    /// let first_to_play = PlayerId::<{TressetteRules::PLAYERS}>::new(0).unwrap();
    /// let card = TressetteCard::new(ItalianRank::Ace, Suit::Hearts);
    /// let mut trick = OngoingTrick::<TressetteRules>::new(first_to_play);
    /// trick.play(card);
    /// let mut second_to_play = first_to_play;
    /// second_to_play.inc();
    ///
    /// assert_eq!(trick[0], Some(card));
    /// assert_eq!(trick.next_to_play(), second_to_play)
    /// ```
    pub fn play(&mut self, card: G::CardType) {
        self.cards[self.next_to_play.0] = Some(card);
        self.next_to_play.inc();
        self.play_count += 1;
    }

    /// Tries to transform the current `OngoingTrick` into a `Trick` by
    /// determining the taker of the trick. It doesn't make any assumption on
    /// previously played cards during the current `OngoingHand`. It also does
    /// not check if it contains duplicates since that could be valid in some games.
    ///
    /// # Errors
    ///
    /// Fails if any of the moves of the `OngoingTrick` this is called on is
    /// None. It means that not all players made their move yet, so a taker
    /// can't be determined.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(generic_const_exprs)]
    /// use shuftlib::common::{hands::{OngoingTrick, PlayerId, TrickTakingGame}, cards::{ItalianRank, Suit}};
    /// use shuftlib::tressette::{TressetteRules, TressetteCard};
    ///
    /// let cards = [
    ///   TressetteCard::new(ItalianRank::Ace, Suit::Hearts),
    ///   TressetteCard::new(ItalianRank::Two, Suit::Hearts),
    ///   TressetteCard::new(ItalianRank::Three, Suit::Hearts),
    ///   TressetteCard::new(ItalianRank::Four, Suit::Hearts),
    /// ];
    /// let first_to_play = PlayerId::<{TressetteRules::PLAYERS}>::new(0).unwrap();
    /// let mut ongoing_trick = OngoingTrick::<TressetteRules>::new(first_to_play);
    /// ongoing_trick.play(cards[0]);
    ///
    /// // After only playing a card, it's not possible to finish the OngoingTrick.
    /// assert!(ongoing_trick.clone().finish().is_none());
    ///
    /// let mut to_play = first_to_play;
    /// to_play.inc();
    /// cards.iter().skip(1).for_each(|&c| {
    ///   ongoing_trick.play(c);
    ///   to_play.inc();
    /// });
    ///
    /// // After every player made their play, it's possible to get the trick.
    /// let trick = ongoing_trick.finish().unwrap();
    /// // Finishing the trick also means determining a taker. Since in this
    /// // example we are using the tressette game rules, player 2 is the taker.
    /// assert_eq!(Some(trick.taker()), PlayerId::<{TressetteRules::PLAYERS}>::new(2));
    /// ```
    pub fn finish(self) -> Option<Trick<G>> {
        let mut cards: [G::CardType; G::PLAYERS] = [G::CardType::default(); G::PLAYERS];
        if self
            .iter()
            .enumerate()
            .map(|(i, &x)| {
                if let Some(c) = x {
                    cards[i] = c;
                    true
                } else {
                    false
                }
            })
            .any(|is_some| !is_some)
        {
            return None;
        }

        let taker = G::determine_taker(&cards, self.first_to_play);
        Some(Trick { cards, taker })
    }

    /// Getter for the cards contained in this `OngoingTrick`.
    pub fn cards(&self) -> &[Option<G::CardType>] {
        &self.cards
    }

    /// Getter for the id of the person who starts the trick.
    pub fn first_to_play(&self) -> PlayerId<{ G::PLAYERS }> {
        self.first_to_play
    }

    /// Getter for the id of the person who playes last in the trick.
    pub fn next_to_play(&self) -> PlayerId<{ G::PLAYERS }> {
        self.next_to_play
    }

    /// Creates a new `OngoingTrick`, by defining the logic to determine the
    /// taker.
    ///
    /// # Examples.
    ///
    /// ```
    /// use shuftlib::common::hands::{OngoingTrick, PlayerId, TrickTakingGame};
    /// use shuftlib::tressette::TressetteRules;
    ///
    /// let first_to_play = PlayerId::<{TressetteRules::PLAYERS}>::new(0).unwrap();
    /// let ongoing_trick = OngoingTrick::<TressetteRules>::new(first_to_play);
    ///
    /// assert_eq!(ongoing_trick.first_to_play(), first_to_play);
    /// ongoing_trick.cards().iter().for_each(|&c| assert!(c.is_none()));
    /// ```
    pub fn new(first_to_play: PlayerId<{ G::PLAYERS }>) -> Self {
        let mut last_to_play = first_to_play;
        (0..G::PLAYERS - 1).for_each(|_| last_to_play.inc());
        Self {
            cards: [None; G::PLAYERS],
            first_to_play,
            next_to_play: first_to_play,
            play_count: 0,
        }
    }
}

/// Various games are usually played multiple times, until one team reaches a
/// certain score. These "multiple times" are called hands: "We played a game of
/// tressette and our team won in just 2 hands!". This type is generic over the
/// actual card type, the number of players allowed and the number of tricks it
/// takes to finish the hand.
#[derive(Debug, Clone, Copy)]
pub struct Hand<G>
where
    G: TrickTakingGame,
    [(); G::PLAYERS]:,
    [(); G::TRICKS]:,
{
    tricks: [Trick<G>; G::TRICKS],
}

impl<G> Hand<G>
where
    G: TrickTakingGame,
    [(); G::PLAYERS]:,
    [(); G::TRICKS]:,
{
    /// Returns a reference to the tricks of this [`Hand<G>`].
    pub fn tricks(&self) -> &[Trick<G>; G::TRICKS] {
        &self.tricks
    }
}

/// A hand takes multiple turns for each player to be completed, this is the
/// representation of a `Hand` which hasn't been completed yet.
#[derive(Clone, Copy, Debug)]
pub struct OngoingHand<G>
where
    G: TrickTakingGame,
    [(); G::PLAYERS]:,
    [(); G::TRICKS]:,
{
    current_trick: Option<OngoingTrick<G>>,
    index: usize,
    tricks: [Option<Trick<G>>; G::TRICKS],
}

impl<G> OngoingHand<G>
where
    G: TrickTakingGame,
    [(); G::PLAYERS]:,
    [(); G::TRICKS]:,
{
    /// Returns the current trick of this [`OngoingHand<G>`].
    pub fn current_trick(&self) -> &Option<OngoingTrick<G>> {
        &self.current_trick
    }

    /// Returns a reference to the tricks of this [`OngoingHand<G>`].
    pub fn tricks(&self) -> &[Option<Trick<G>>; G::TRICKS] {
        &self.tricks
    }

    /// Returns the index of this [`OngoingHand<G>`].
    pub fn index(&self) -> usize {
        self.index
    }

    /// Transforms an `OngoingHand` into a `Hand`, a read-only data structure
    /// used to just story the information related to a hand that has been played.
    pub fn finish(self) -> Option<Hand<G>> {
        if self.tricks.iter().any(|t| t.is_none()) {
            return None;
        }

        let tricks: [Trick<G>; G::TRICKS] = self
            .tricks
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .try_into()
            .ok()?;
        Some(Hand { tricks })
    }

    /// Constructor for `OngoingHand`. All the internal fields are initialized
    /// as empty or None.
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::{common::{hands::OngoingHand}, tressette::TressetteRules};
    ///
    /// let ongoing_hand = OngoingHand::<TressetteRules>::new();
    ///
    /// assert_eq!(ongoing_hand.index(), 0);
    /// assert!(ongoing_hand.current_trick().is_none());
    /// ongoing_hand.tricks().iter().for_each(|t| assert!(t.is_none()));
    /// ```
    pub fn new() -> Self {
        let tricks: [Option<Trick<G>>; G::TRICKS] = array_init::array_init(|_| None);

        let current_trick = None;
        let index = 0;

        Self {
            tricks,
            current_trick,
            index,
        }
    }

    /// Adds a trick to this hand.
    pub fn add(&mut self, trick: Trick<G>, id: usize) {
        self.tricks[id] = Some(trick);
    }
}

impl<G> Default for OngoingHand<G>
where
    G: TrickTakingGame,
    [(); G::PLAYERS]:,
    [(); G::TRICKS]:,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use proptest::collection::hash_set;
    use proptest::{array, prelude::*};

    use crate::common::cards::{ItalianCard, ItalianRank, Suit};

    use super::{OngoingTrick, PlayerId, TrickTakingGame};

    /// Strategy to create a random `TressetteCard`.
    fn italian_card_strategy() -> impl Strategy<Value = ItalianCard> {
        (
            prop_oneof![
                Just(ItalianRank::Ace),
                Just(ItalianRank::Two),
                Just(ItalianRank::Three),
                Just(ItalianRank::Four),
                Just(ItalianRank::Five),
                Just(ItalianRank::Six),
                Just(ItalianRank::Seven),
                Just(ItalianRank::Jack),
                Just(ItalianRank::Knight),
                Just(ItalianRank::King),
            ],
            prop_oneof![
                Just(Suit::Hearts),
                Just(Suit::Clubs),
                Just(Suit::Spades),
                Just(Suit::Diamonds),
            ],
        )
            .prop_map(|(rank, suit)| ItalianCard::new(rank, suit))
    }

    #[derive(Clone, Copy, Debug)]
    struct TestGame {}

    impl TrickTakingGame for TestGame {
        type CardType = ItalianCard;

        const PLAYERS: usize = 4;

        const TRICKS: usize = 10;

        fn determine_taker(
            _cards: &[Self::CardType; Self::PLAYERS],
            _first_to_play: super::PlayerId<{ Self::PLAYERS }>,
        ) -> super::PlayerId<{ Self::PLAYERS }> {
            PlayerId::new(0).unwrap()
        }
    }

    /// Strategy to create an `OngoingTrick` filled with random cards. Since
    /// the `OngoingTrick` already contains the cards, `first_to_play` is
    /// irrelevant. Change this function accordingly if you need those to have
    /// a specific value.
    fn ongoing_trick_strategy() -> impl Strategy<Value = OngoingTrick<TestGame>> {
        hash_set(italian_card_strategy(), TestGame::PLAYERS).prop_map(|hash_set| {
            let mut cards = [None; TestGame::PLAYERS];
            hash_set
                .iter()
                .enumerate()
                .for_each(|(i, &c)| cards[i] = Some(c));

            OngoingTrick {
                cards,
                first_to_play: PlayerId(0),
                next_to_play: PlayerId(0),
                play_count: 0,
            }
        })
    }

    proptest! {
        #[test]
        fn play_method_works(cards in array::uniform4(italian_card_strategy())) {
            let mut trick: OngoingTrick<TestGame> = OngoingTrick::new(PlayerId::new(0).unwrap());

            for (index, &card) in cards.iter().enumerate() {
                // Panicking if there are duplicates in the cards array.
                trick.play(card);
                // If the card was successfully played, it will be contained
                // inside the `OngoingTrick` struct as `Some`.
                assert_eq!(trick[index], Some(card));
            }
        }

        #[test]
        fn finish_method_works(ongoing_trick in ongoing_trick_strategy()) {
            let trick = ongoing_trick.finish().unwrap();

            let cards = ongoing_trick.cards();

            prop_assert_eq!(trick.taker(), PlayerId::new(0).unwrap());
            prop_assert_eq!(trick.taken_with(), cards[0].unwrap());
        }
    }
}
