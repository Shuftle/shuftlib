use std::fmt::Display;

use anyhow::bail;

use crate::trick_taking::TrickTakingGame;

/// Number of players in all games.
pub const PLAYERS: usize = 4;

/// Represents a player of a game. This type is generic over the type of the
/// card used for the specific game.
#[derive(Clone, Default, Debug)]
pub struct Player<G>
where
    G: TrickTakingGame,
{
    /// The cards held in the player's hand.
    hand: Vec<G::CardType>,
    /// The ID of this player.
    id: PlayerId,
}

impl<G> Player<G>
where
    G: TrickTakingGame,
{
    /// Adds a card to the hand of the player.
    ///
    /// # Examples
    /// ```
    /// use shuftlib::trick_taking::{Player, TrickTakingGame, PlayerId};
    /// use shuftlib::core::Suit;
    /// use shuftlib::core::italian::ItalianRank;
    /// use shuftlib::tressette::{TressetteRules, TressetteCard};
    ///
    /// let player_id = PlayerId::PLAYER_0;
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

    /// Removes a card at the specified index from the player's hand.
    ///
    /// Returns `Some(card)` if the index is valid, `None` otherwise.
    /// The hand is not modified if the index is out of bounds.
    ///
    /// # Examples
    /// ```
    /// use shuftlib::trick_taking::{Player, TrickTakingGame, PlayerId};
    /// use shuftlib::core::Suit;
    /// use shuftlib::core::italian::ItalianRank;
    /// use shuftlib::tressette::{TressetteRules, TressetteCard};
    ///
    /// let player_id = PlayerId::PLAYER_0;
    /// let mut player = Player::<TressetteRules>::new(player_id);
    ///
    /// let card1 = TressetteCard::new(ItalianRank::Ace, Suit::Spades);
    /// let card2 = TressetteCard::new(ItalianRank::King, Suit::Hearts);
    /// player.give(card1);
    /// player.give(card2);
    ///
    /// // Remove card at index 0
    /// let removed = player.remove_card(0);
    /// assert_eq!(removed, Some(card1));
    /// assert_eq!(player.hand().len(), 1);
    ///
    /// // Try to remove at invalid index
    /// let removed = player.remove_card(10);
    /// assert_eq!(removed, None);
    /// assert_eq!(player.hand().len(), 1);
    /// ```
    pub fn remove_card(&mut self, index: usize) -> Option<G::CardType> {
        if index < self.hand.len() {
            Some(self.hand.remove(index))
        } else {
            None
        }
    }

    /// Returns the cards held by this player.
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::trick_taking::{Player, PlayerId};
    /// use shuftlib::tressette::TressetteRules;
    ///
    /// let player = Player::<TressetteRules>::new(PlayerId::PLAYER_0);
    /// assert_eq!(player.hand().len(), 0);
    /// ```
    pub fn hand(&self) -> &[G::CardType] {
        &self.hand
    }

    /// Returns the ID of this player.
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::trick_taking::{Player, PlayerId};
    /// use shuftlib::tressette::TressetteRules;
    ///
    /// let player = Player::<TressetteRules>::new(PlayerId::PLAYER_0);
    /// assert_eq!(player.id(), PlayerId::PLAYER_0);
    /// ```
    pub fn id(&self) -> PlayerId {
        self.id
    }

    /// Generates a new player from a `PlayerId`. Players are initialized with
    /// no cards.
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::{trick_taking::{Player, PlayerId, TrickTakingGame}, tressette::TressetteRules};
    ///
    /// let id = PlayerId::PLAYER_0;
    /// let player = Player::<TressetteRules>::new(id);
    ///
    /// assert_eq!(player.id().as_usize(), 0);
    /// assert_eq!(player.hand().len(), 0);
    /// ```
    pub fn new(id: PlayerId) -> Self {
        Self {
            id,
            hand: Vec::new(),
        }
    }
}

/// A player id can only be in the range 0..4.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct PlayerId(usize);

impl PlayerId {
    /// Player ID constant for player 0.
    pub const PLAYER_0: Self = PlayerId(0);
    /// Player ID constant for player 1.
    pub const PLAYER_1: Self = PlayerId(1);
    /// Player ID constant for player 2.
    pub const PLAYER_2: Self = PlayerId(2);
    /// Player ID constant for player 3.
    pub const PLAYER_3: Self = PlayerId(3);

    /// This method simply increments `self` by 1. Note that `PlayerId` can only
    /// be in the range 0..N, so incrementing `self` when the value is N-1, will
    /// reset its value to 0, since the purpose of this type is to determine the
    /// player's turn and the first person to play is not necessarily the person
    /// with ID = 0.
    ///
    /// # Examples
    /// ```
    /// use shuftlib::trick_taking::PlayerId;
    ///
    /// let mut player_id = PlayerId::PLAYER_0;
    /// player_id.inc();
    /// assert_eq!(player_id, PlayerId::PLAYER_1);
    /// player_id.inc();
    /// player_id.inc();
    /// player_id.inc();
    /// assert_eq!(player_id, PlayerId::PLAYER_0);
    /// ```
    pub fn inc(&mut self) {
        if self.0 < PLAYERS - 1 {
            self.0 += 1;
        } else {
            self.0 = 0;
        }
    }

    /// Returns the next player ID by incrementing this one.
    /// Note that `PlayerId` wraps around from N-1 to 0.
    ///
    /// # Examples
    /// ```
    /// use shuftlib::trick_taking::PlayerId;
    ///
    /// let player_id = PlayerId::PLAYER_0;
    /// assert_eq!(player_id.next(), PlayerId::PLAYER_1);
    ///
    /// let last_player = PlayerId::PLAYER_3;
    /// assert_eq!(last_player.next(), PlayerId::PLAYER_0);
    /// ```
    pub fn next(mut self) -> Self {
        self.inc();
        self
    }

    /// Returns the underlying usize value of this PlayerId.
    ///
    /// # Examples
    /// ```
    /// use shuftlib::trick_taking::PlayerId;
    ///
    /// let player_id = PlayerId::PLAYER_2;
    /// assert_eq!(player_id.as_usize(), 2);
    /// ```
    pub fn as_usize(&self) -> usize {
        self.0
    }
}

impl TryFrom<usize> for PlayerId {
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

impl Display for PlayerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
