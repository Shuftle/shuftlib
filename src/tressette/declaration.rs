//! Declarations (accusi) of Tressette: the napoli, and sets of three or four
//! cards of one rank.
//!
//! What counts as a declaration, and when it may be made, depends on the
//! regional variant, so the timing and visibility are a [`Rules`] value rather
//! than fixed behaviour.

use crate::cards::italian::{ItalianRank, Suit};

use super::TressetteCard;

/// Points for a napoli: the Ace, Two and Three of one suit.
pub const NAPOLI_POINTS: u8 = 3;
/// Points for three cards of one rank.
pub const THREE_OF_A_KIND_POINTS: u8 = 3;
/// Points for four cards of one rank.
pub const FOUR_OF_A_KIND_POINTS: u8 = 4;

/// The ranks that can be declared as a set.
pub const DECLARABLE_RANKS: [ItalianRank; 3] =
    [ItalianRank::Three, ItalianRank::Two, ItalianRank::Ace];

/// The ranks that make up a napoli, in any suit.
const NAPOLI_RANKS: [ItalianRank; 3] = [ItalianRank::Ace, ItalianRank::Two, ItalianRank::Three];

/// The number of cards in an accuso: three or four of one rank.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Size {
    /// Three cards of one rank.
    Three = 3,
    /// Four cards of one rank.
    Four = 4,
}

/// A declaration a seat can make.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Declaration {
    /// The Ace, Two and Three of one suit.
    Napoli(Suit),
    /// Three or four cards of one declarable rank.
    Accuso {
        /// The rank of the cards.
        rank: ItalianRank,
        /// How many cards of that rank.
        size: Size,
    },
}

impl Declaration {
    /// Returns the points this declaration is worth.
    pub fn points(&self) -> u8 {
        match self {
            Self::Napoli(_) => NAPOLI_POINTS,
            Self::Accuso {
                size: Size::Three, ..
            } => THREE_OF_A_KIND_POINTS,
            Self::Accuso {
                size: Size::Four, ..
            } => FOUR_OF_A_KIND_POINTS,
        }
    }

    /// Returns whether `hand` holds every card this declaration needs, and
    /// the rank is one that can be declared.
    pub fn can_declare(&self, hand: &[TressetteCard]) -> bool {
        match *self {
            Self::Napoli(suit) => NAPOLI_RANKS
                .iter()
                .all(|&rank| hand.contains(&TressetteCard::new(rank, suit))),
            Self::Accuso { rank, size } => {
                DECLARABLE_RANKS.contains(&rank)
                    && hand.iter().filter(|card| card.rank() == rank).count() >= size as usize
            }
        }
    }
}

/// The turns of a hand on which a declaration may be made.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Window {
    /// Only on the first trick.
    FirstTrick,
    /// On any of the first three tricks.
    FirstThreeTricks,
}

/// The last trick on which [`Window::FirstThreeTricks`] allows a declaration.
const LAST_TRICK_OF_FIRST_THREE: usize = 3;

impl Window {
    /// Returns whether a declaration may be made on the 1-based `trick`.
    pub fn allows(self, trick: usize) -> bool {
        match self {
            Self::FirstTrick => trick == 1,
            Self::FirstThreeTricks => trick <= LAST_TRICK_OF_FIRST_THREE,
        }
    }
}

/// What other seats learn when a declaration is made.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Visibility {
    /// Everyone learns that a declaration was made, but not which one.
    Anonymous,
    /// Everyone learns the exact declaration.
    Exact,
}

/// The declaration rules of one regional variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rules {
    /// The tricks on which a declaration may be made.
    pub window: Window,
    /// What other seats learn from a declaration.
    pub visibility: Visibility,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::italian::test_utils::{italian_rank_strategy, suit_strategy};
    use crate::tressette::HAND_SIZE;
    use proptest::{prelude::*, sample};

    fn all_cards() -> Vec<TressetteCard> {
        ItalianRank::ALL
            .iter()
            .flat_map(|&rank| {
                Suit::ALL
                    .iter()
                    .map(move |&suit| TressetteCard::new(rank, suit))
            })
            .collect()
    }

    fn hand_strategy() -> impl Strategy<Value = Vec<TressetteCard>> {
        let deck = all_cards();
        let size = deck.len();
        sample::subsequence(deck, 0..=size)
    }

    fn card_strategy() -> impl Strategy<Value = TressetteCard> {
        sample::select(all_cards())
    }

    /// Returns `hand` with every card of `cards` it does not already hold.
    fn with_cards(hand: &[TressetteCard], cards: &[TressetteCard]) -> Vec<TressetteCard> {
        let mut larger = hand.to_vec();
        for &card in cards {
            if !larger.contains(&card) {
                larger.push(card);
            }
        }
        larger
    }

    /// The napoli cards of `suit`.
    fn napoli_cards(suit: Suit) -> Vec<TressetteCard> {
        NAPOLI_RANKS
            .iter()
            .map(|&rank| TressetteCard::new(rank, suit))
            .collect()
    }

    /// The first `count` cards of `rank`, one per suit.
    fn cards_of_rank(rank: ItalianRank, count: usize) -> Vec<TressetteCard> {
        Suit::ALL
            .iter()
            .take(count)
            .map(|&suit| TressetteCard::new(rank, suit))
            .collect()
    }

    fn copies_of(hand: &[TressetteCard], rank: ItalianRank) -> usize {
        hand.iter().filter(|card| card.rank() == rank).count()
    }

    proptest! {
        /// Any hand that holds a set can declare it, whatever else it holds.
        #[test]
        fn a_hand_holding_the_set_can_declare_it(
            suit in suit_strategy(),
            rank in sample::select(DECLARABLE_RANKS.to_vec()),
            hand in hand_strategy(),
        ) {
            prop_assert!(Declaration::Napoli(suit).can_declare(&with_cards(&hand, &napoli_cards(suit))));
            let three = cards_of_rank(rank, Size::Three as usize);
            let declaration = Declaration::Accuso { rank, size: Size::Three };
            prop_assert!(declaration.can_declare(&with_cards(&hand, &three)));
            let four = cards_of_rank(rank, Size::Four as usize);
            let declaration = Declaration::Accuso { rank, size: Size::Four };
            prop_assert!(declaration.can_declare(&with_cards(&hand, &four)));
        }

        /// The napoli is declarable exactly when the hand holds the Ace, Two
        /// and Three of its suit.
        #[test]
        fn napoli_is_declarable_exactly_when_the_run_is_held(
            suit in suit_strategy(),
            hand in hand_strategy(),
        ) {
            let held = napoli_cards(suit).iter().all(|card| hand.contains(card));
            prop_assert_eq!(Declaration::Napoli(suit).can_declare(&hand), held);
        }

        /// Three of a kind is declarable exactly when the rank is Three, Two
        /// or Ace and the hand holds at least three cards of it.
        #[test]
        fn three_of_a_kind_needs_three_cards_of_a_declarable_rank(
            rank in italian_rank_strategy(),
            hand in hand_strategy(),
        ) {
            let expected = DECLARABLE_RANKS.contains(&rank)
                && copies_of(&hand, rank) >= Size::Three as usize;
            let declaration = Declaration::Accuso { rank, size: Size::Three };
            prop_assert_eq!(declaration.can_declare(&hand), expected);
        }

        /// Four of a kind is declarable exactly when the rank is Three, Two
        /// or Ace and the hand holds all four cards of it.
        #[test]
        fn four_of_a_kind_needs_four_cards_of_a_declarable_rank(
            rank in italian_rank_strategy(),
            hand in hand_strategy(),
        ) {
            let expected = DECLARABLE_RANKS.contains(&rank)
                && copies_of(&hand, rank) >= Size::Four as usize;
            let declaration = Declaration::Accuso { rank, size: Size::Four };
            prop_assert_eq!(declaration.can_declare(&hand), expected);
        }

        /// Adding a card to a hand that holds a set never removes the ability
        /// to declare it.
        #[test]
        fn adding_a_card_never_removes_a_declaration(
            suit in suit_strategy(),
            rank in sample::select(DECLARABLE_RANKS.to_vec()),
            hand in hand_strategy(),
            extra in card_strategy(),
        ) {
            let three = cards_of_rank(rank, Size::Three as usize);
            let four = cards_of_rank(rank, Size::Four as usize);
            for (declaration, set) in [
                (Declaration::Napoli(suit), napoli_cards(suit)),
                (Declaration::Accuso { rank, size: Size::Three }, three),
                (Declaration::Accuso { rank, size: Size::Four }, four),
            ] {
                let larger = with_cards(&with_cards(&hand, &set), &[extra]);
                prop_assert!(declaration.can_declare(&larger));
            }
        }

        /// The points depend only on the kind of declaration.
        #[test]
        fn points_depend_only_on_the_kind(suit in suit_strategy(), rank in italian_rank_strategy()) {
            prop_assert_eq!(Declaration::Napoli(suit).points(), NAPOLI_POINTS);
            prop_assert_eq!(
                Declaration::Accuso { rank, size: Size::Three }.points(),
                THREE_OF_A_KIND_POINTS
            );
            prop_assert_eq!(
                Declaration::Accuso { rank, size: Size::Four }.points(),
                FOUR_OF_A_KIND_POINTS
            );
        }

        /// The first-trick window allows exactly trick 1.
        #[test]
        fn first_trick_window_allows_only_trick_one(trick in 1..=HAND_SIZE) {
            prop_assert_eq!(Window::FirstTrick.allows(trick), trick == 1);
        }

        /// The first-three-tricks window allows exactly tricks 1 to 3.
        #[test]
        fn first_three_tricks_window_allows_tricks_one_to_three(trick in 1..=HAND_SIZE) {
            prop_assert_eq!(
                Window::FirstThreeTricks.allows(trick),
                trick <= LAST_TRICK_OF_FIRST_THREE
            );
        }
    }
}
