//! The table: four seats in two partnerships.

use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};

/// A seat at the table. Partners sit opposite each other: North with South,
/// East with West. Play goes counter-clockwise, as is customary for Italian
/// games: North, West, South, East.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    /// ```
    /// use shuftlib::tressette::Seat;
    ///
    /// assert_eq!(Seat::ALL, [Seat::North, Seat::West, Seat::South, Seat::East]);
    /// ```
    pub const ALL: [Self; 4] = [Self::North, Self::West, Self::South, Self::East];

    /// Returns the seat that plays after this one.
    ///
    /// ```
    /// use shuftlib::tressette::Seat;
    ///
    /// assert_eq!(Seat::North.next(), Seat::West);
    /// assert_eq!(Seat::East.next(), Seat::North);
    /// ```
    pub const fn next(self) -> Self {
        self.after(1)
    }

    /// Returns the seat `steps` turns after this one.
    ///
    /// ```
    /// use shuftlib::tressette::Seat;
    ///
    /// assert_eq!(Seat::East.after(2), Seat::West);
    /// assert_eq!(Seat::East.after(4), Seat::East);
    /// ```
    pub const fn after(self, steps: usize) -> Self {
        // Reducing `steps` first keeps the sum from overflowing.
        Self::ALL[(self.position() + steps % Self::ALL.len()) % Self::ALL.len()]
    }

    /// Returns the team this seat plays for.
    ///
    /// ```
    /// use shuftlib::tressette::{Seat, Team};
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

    /// Returns the seat's position in [`ALL`](Self::ALL).
    const fn position(self) -> usize {
        match self {
            Self::North => 0,
            Self::West => 1,
            Self::South => 2,
            Self::East => 3,
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
    /// ```
    /// use shuftlib::tressette::Team;
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

/// A value for each seat, such as how many cards each seat holds: a
/// `PerSeat<usize>`. Read one through its field, `hand_sizes.east`, or by
/// indexing with a [`Seat`], `hand_sizes[seat]`.
///
/// ```
/// use shuftlib::tressette::{PerSeat, Seat};
///
/// let mut hand_sizes = PerSeat::from_fn(|_| 10);
/// // East plays a card.
/// hand_sizes[Seat::East] -= 1;
/// assert_eq!(hand_sizes.east, 9);
/// assert_eq!(hand_sizes[Seat::West], 10);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct PerSeat<T> {
    /// North's value.
    pub north: T,
    /// East's value.
    pub east: T,
    /// South's value.
    pub south: T,
    /// West's value.
    pub west: T,
}

impl<T> PerSeat<T> {
    /// Builds the values by calling `value` for each [`Seat`], in play order
    /// from North, as in [`Seat::ALL`].
    pub fn from_fn(mut value: impl FnMut(Seat) -> T) -> Self {
        let north = value(Seat::North);
        let west = value(Seat::West);
        let south = value(Seat::South);
        let east = value(Seat::East);
        Self {
            north,
            east,
            south,
            west,
        }
    }
}

impl<T> Index<Seat> for PerSeat<T> {
    type Output = T;

    fn index(&self, seat: Seat) -> &T {
        match seat {
            Seat::North => &self.north,
            Seat::East => &self.east,
            Seat::South => &self.south,
            Seat::West => &self.west,
        }
    }
}

impl<T> IndexMut<Seat> for PerSeat<T> {
    fn index_mut(&mut self, seat: Seat) -> &mut T {
        match seat {
            Seat::North => &mut self.north,
            Seat::East => &mut self.east,
            Seat::South => &mut self.south,
            Seat::West => &mut self.west,
        }
    }
}

/// A value for each team, such as the score: a `PerTeam<u8>`. Read one
/// through its field, `score.north_south`, or by indexing with a [`Team`],
/// `score[team]`, which also works with a seat's team: `score[seat.team()]`.
///
/// ```
/// use shuftlib::tressette::{PerTeam, Seat, Team};
///
/// let mut score = PerTeam::<u8>::default();
/// // West's team scores 3 points.
/// score[Seat::West.team()] += 3;
/// assert_eq!(score.east_west, 3);
/// assert_eq!(score[Team::NorthSouth], 0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct PerTeam<T> {
    /// North and South's value.
    pub north_south: T,
    /// East and West's value.
    pub east_west: T,
}

impl<T> PerTeam<T> {
    /// Builds the values by calling `value` for each [`Team`].
    pub fn from_fn(mut value: impl FnMut(Team) -> T) -> Self {
        Self {
            north_south: value(Team::NorthSouth),
            east_west: value(Team::EastWest),
        }
    }
}

impl<T> Index<Team> for PerTeam<T> {
    type Output = T;

    fn index(&self, team: Team) -> &T {
        match team {
            Team::NorthSouth => &self.north_south,
            Team::EastWest => &self.east_west,
        }
    }
}

impl<T> IndexMut<Team> for PerTeam<T> {
    fn index_mut(&mut self, team: Team) -> &mut T {
        match team {
            Team::NorthSouth => &mut self.north_south,
            Team::EastWest => &mut self.east_west,
        }
    }
}

#[cfg(test)]
/// Test utilities for the Tressette table module.
pub mod test_utils {
    use super::Seat;
    use proptest::{prelude::*, sample};

    /// Generates any seat.
    pub fn seat_strategy() -> impl Strategy<Value = Seat> {
        sample::select(Seat::ALL.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::{PerSeat, Seat, test_utils::seat_strategy};

    #[test]
    fn values_for_each_seat_are_built_in_play_order() {
        let mut visited = Vec::new();
        let built = PerSeat::from_fn(|seat| {
            visited.push(seat);
            visited.len()
        });
        assert_eq!(visited, Seat::ALL);
        assert_eq!(built[Seat::West], 2);
    }

    #[test]
    fn play_order_goes_round_the_table() {
        for (position, seat) in Seat::ALL.into_iter().enumerate() {
            assert_eq!(seat.next(), Seat::ALL[(position + 1) % Seat::ALL.len()]);
        }
    }

    /// Even the largest numbers of turns land where the leftover turns,
    /// after whole rounds of the table, lead.
    #[test]
    fn after_goes_round_the_table_however_far() {
        for seat in Seat::ALL {
            for back in 0..8 {
                let steps = usize::MAX - back;
                assert_eq!(
                    seat.after(steps),
                    seat.after(steps % 4),
                    "{seat} after {steps}"
                );
            }
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
        fn after_takes_that_many_turns(
            seat in seat_strategy(),
            steps in 0usize..100,
        ) {
            let walked = (0..steps).fold(seat, |seat, _| seat.next());
            prop_assert_eq!(seat.after(steps), walked);
        }
    }
}
