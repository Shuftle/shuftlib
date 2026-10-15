//! Player identification and per-player hand management.
//!
//! [`Seat`] names one of the [`PLAYERS`] seats at the table, and [`Team`] one of
//! the two partnerships; [`Player`] holds the cards currently in a seat's hand.

use std::fmt::Display;

use crate::trick_taking::TrickTakingGame;

/// Number of players at the table.
pub const PLAYERS: usize = 4;

/// Represents a player of a game. This type is generic over the type of the
/// card used for the specific game.
#[derive(Clone, Debug)]
pub struct Player<G>
where
    G: TrickTakingGame,
{
    /// The cards held in the player's hand.
    hand: Vec<G::CardType>,
    /// The ID of this player.
    id: Seat,
}

impl<G> Player<G>
where
    G: TrickTakingGame,
{
    /// Adds a card to the hand of the player.
    ///
    /// # Examples
    /// ```
    /// use shuftlib::{trick_taking::{Player, TrickTakingGame, Seat}, cards::italian::{ItalianRank, Suit}, tressette::{TressetteRules, TressetteCard}};
    ///
    /// let seat = Seat::North;
    /// let mut player = Player::<TressetteRules>::new(seat);
    /// // Players have no cards when created.
    /// assert_eq!(player.hand().len(), 0);
    ///
    /// let card = TressetteCard::new(ItalianRank::Ace, Suit::Swords);
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
    /// use shuftlib::{trick_taking::{Player, TrickTakingGame, Seat}, cards::italian::{ItalianRank, Suit}, tressette::{TressetteRules, TressetteCard}};
    ///
    /// let seat = Seat::North;
    /// let mut player = Player::<TressetteRules>::new(seat);
    ///
    /// let card1 = TressetteCard::new(ItalianRank::Ace, Suit::Swords);
    /// let card2 = TressetteCard::new(ItalianRank::King, Suit::Cups);
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
    /// use shuftlib::{trick_taking::{Player, Seat}, tressette::TressetteRules};
    ///
    /// let player = Player::<TressetteRules>::new(Seat::North);
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
    /// use shuftlib::{trick_taking::{Player, Seat}, tressette::TressetteRules};
    ///
    /// let player = Player::<TressetteRules>::new(Seat::North);
    /// assert_eq!(player.id(), Seat::North);
    /// ```
    pub fn id(&self) -> Seat {
        self.id
    }

    /// Generates a new player from a `Seat`. Players are initialized with
    /// no cards.
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::{trick_taking::{Player, Seat, TrickTakingGame}, tressette::TressetteRules};
    ///
    /// let id = Seat::North;
    /// let player = Player::<TressetteRules>::new(id);
    ///
    /// assert_eq!(player.id().index(), 0);
    /// assert_eq!(player.hand().len(), 0);
    /// ```
    pub fn new(id: Seat) -> Self {
        Self {
            id,
            hand: Vec::new(),
        }
    }
}

/// A seat at the table. Partners sit opposite each other: North with South,
/// East with West. Play goes counter-clockwise, as is customary for Italian
/// games: North, West, South, East.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Seat {
    /// Partner of South.
    North,
    /// Partner of West.
    East,
    /// Partner of North.
    South,
    /// Partner of East.
    West,
}

impl Seat {
    /// Every seat, in play order from North.
    ///
    /// # Examples
    /// ```
    /// use shuftlib::trick_taking::Seat;
    ///
    /// assert_eq!(Seat::ALL, [Seat::North, Seat::West, Seat::South, Seat::East]);
    /// ```
    pub const ALL: [Self; PLAYERS] = [Self::North, Self::West, Self::South, Self::East];

    /// Returns the seat that plays after this one.
    ///
    /// # Examples
    /// ```
    /// use shuftlib::trick_taking::Seat;
    ///
    /// assert_eq!(Seat::North.next(), Seat::West);
    /// assert_eq!(Seat::East.next(), Seat::North);
    /// ```
    pub const fn next(self) -> Self {
        match self {
            Self::North => Self::West,
            Self::West => Self::South,
            Self::South => Self::East,
            Self::East => Self::North,
        }
    }

    /// Returns the seat `steps` turns after this one.
    ///
    /// # Examples
    /// ```
    /// use shuftlib::trick_taking::Seat;
    ///
    /// assert_eq!(Seat::East.after(2), Seat::West);
    /// assert_eq!(Seat::East.after(4), Seat::East);
    /// ```
    pub fn after(self, steps: usize) -> Self {
        Self::ALL[(self.index() + steps) % PLAYERS]
    }

    /// Returns the seat's position in [`ALL`](Self::ALL), the play order from
    /// North. Values kept for each seat are stored in this order.
    ///
    /// # Examples
    /// ```
    /// use shuftlib::trick_taking::Seat;
    ///
    /// assert_eq!(Seat::North.index(), 0);
    /// assert_eq!(Seat::West.index(), 1);
    /// ```
    pub const fn index(self) -> usize {
        match self {
            Self::North => 0,
            Self::West => 1,
            Self::South => 2,
            Self::East => 3,
        }
    }

    /// Returns the team this seat plays for.
    ///
    /// # Examples
    /// ```
    /// use shuftlib::trick_taking::{Seat, Team};
    ///
    /// assert_eq!(Seat::South.team(), Team::NorthSouth);
    /// assert_eq!(Seat::West.team(), Team::EastWest);
    /// ```
    pub const fn team(self) -> Team {
        match self {
            Self::North | Self::South => Team::NorthSouth,
            Self::East | Self::West => Team::EastWest,
        }
    }
}

impl Display for Seat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::North => "North",
            Self::East => "East",
            Self::South => "South",
            Self::West => "West",
        };
        f.write_str(name)
    }
}

/// One of the two partnerships.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Team {
    /// North and South.
    NorthSouth,
    /// East and West.
    EastWest,
}

impl Team {
    /// Both teams.
    pub const ALL: [Self; 2] = [Self::NorthSouth, Self::EastWest];

    /// Returns the other team.
    ///
    /// # Examples
    /// ```
    /// use shuftlib::trick_taking::Team;
    ///
    /// assert_eq!(Team::NorthSouth.opponent(), Team::EastWest);
    /// ```
    pub const fn opponent(self) -> Self {
        match self {
            Self::NorthSouth => Self::EastWest,
            Self::EastWest => Self::NorthSouth,
        }
    }
}

impl Display for Team {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::NorthSouth => "North-South",
            Self::EastWest => "East-West",
        };
        f.write_str(name)
    }
}

#[cfg(test)]
pub(crate) mod test_utils {
    use proptest::{prelude::Strategy, sample};

    use crate::trick_taking::Seat;

    pub(crate) fn seat_strategy() -> impl Strategy<Value = Seat> {
        sample::select(Seat::ALL.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::{Seat, test_utils::seat_strategy};

    #[test]
    fn index_is_the_position_in_play_order() {
        for (index, seat) in Seat::ALL.into_iter().enumerate() {
            assert_eq!(seat.index(), index);
            assert_eq!(seat.next(), Seat::ALL[(index + 1) % Seat::ALL.len()]);
        }
    }

    #[test]
    fn partners_sit_two_turns_apart() {
        for seat in Seat::ALL {
            assert_eq!(seat.after(2).team(), seat.team());
            assert_ne!(seat.next().team(), seat.team());
        }
    }

    proptest! {
        #[test]
        fn after_takes_that_many_turns(seat in seat_strategy(), steps in 0usize..100) {
            let walked = (0..steps).fold(seat, |seat, _| seat.next());
            prop_assert_eq!(seat.after(steps), walked);
        }
    }
}
