//! Declarations (accusi) of Tressette: the napoli, and sets of three or four
//! cards of one rank.
//!
//! What counts as a declaration, and when it may be made, depends on the
//! regional variant, so the timing and visibility are a [`Rules`] value rather
//! than fixed behaviour.

use crate::cards::italian::{ItalianRank, Suit};

use super::{Error, TressetteCard};

/// Points for a napoli: the Ace, Two and Three of one suit.
pub const NAPOLI_POINTS: u8 = 3;
/// Points for three cards of one rank.
pub const THREE_OF_A_KIND_POINTS: u8 = 3;
/// Points for four cards of one rank.
pub const FOUR_OF_A_KIND_POINTS: u8 = 4;

/// A rank that can be declared as a set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeclarableRank {
    /// Threes.
    Three,
    /// Twos.
    Two,
    /// Aces.
    Ace,
}

impl DeclarableRank {
    /// Every rank that can be declared as a set.
    pub const ALL: [Self; 3] = [Self::Three, Self::Two, Self::Ace];
}

impl From<DeclarableRank> for ItalianRank {
    fn from(rank: DeclarableRank) -> Self {
        match rank {
            DeclarableRank::Three => Self::Three,
            DeclarableRank::Two => Self::Two,
            DeclarableRank::Ace => Self::Ace,
        }
    }
}

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
        rank: DeclarableRank,
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

    /// Returns every declaration there is: a napoli per suit, and three and
    /// four of a kind of each declarable rank.
    pub fn all() -> impl Iterator<Item = Self> {
        let napoli = Suit::ALL.into_iter().map(Self::Napoli);
        let accusi = DeclarableRank::ALL
            .into_iter()
            .flat_map(|rank| [Size::Three, Size::Four].map(|size| Self::Accuso { rank, size }));
        napoli.chain(accusi)
    }

    /// Returns whether `hand` holds every card this declaration needs.
    pub fn can_declare(&self, hand: &[TressetteCard]) -> bool {
        match *self {
            Self::Napoli(suit) => NAPOLI_RANKS
                .iter()
                .all(|&rank| hand.contains(&TressetteCard::new(rank, suit))),
            Self::Accuso { rank, size } => {
                let rank = ItalianRank::from(rank);
                hand.iter().filter(|card| card.rank() == rank).count() >= size as usize
            }
        }
    }
}

/// What a seat's declaration is read against.
pub(super) struct Context<'a> {
    /// The tricks on which the variant allows declarations.
    pub rules: Rules,
    /// The 1-based number of the trick being played.
    pub trick: usize,
    /// The seat's cards.
    pub hand: &'a [TressetteCard],
    /// What the seat already declared this hand.
    pub declared: &'a [Declared],
}

/// A declaration a seat has made. It can only be built from a declaration
/// the rules allowed, so every `Declared` was legal when it was made.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Declared(Declaration);

impl Declared {
    /// Reads `declaration` against `context`, or says why it can't be made:
    /// when several reasons apply, the first of
    /// [`WindowClosed`](DeclarationRefusal::WindowClosed),
    /// [`NotHeld`](DeclarationRefusal::NotHeld),
    /// [`AlreadyDeclared`](DeclarationRefusal::AlreadyDeclared) and
    /// [`ReplacedByFour`](DeclarationRefusal::ReplacedByFour).
    pub(super) fn parse(declaration: Declaration, context: &Context) -> Result<Self, Error> {
        let refused = |reason| Error::CannotDeclare {
            declaration,
            reason,
        };
        if !context.rules.window.allows(context.trick) {
            return Err(refused(DeclarationRefusal::WindowClosed));
        }
        if !declaration.can_declare(context.hand) {
            return Err(refused(DeclarationRefusal::NotHeld));
        }
        let already = context
            .declared
            .iter()
            .any(|&Declared(made)| match (declaration, made) {
                (Declaration::Napoli(_), _) => made == declaration,
                (Declaration::Accuso { rank, .. }, Declaration::Accuso { rank: taken, .. }) => {
                    rank == taken
                }
                (Declaration::Accuso { .. }, Declaration::Napoli(_)) => false,
            });
        if already {
            return Err(refused(DeclarationRefusal::AlreadyDeclared));
        }
        if let Declaration::Accuso {
            rank,
            size: Size::Three,
        } = declaration
            && (Declaration::Accuso {
                rank,
                size: Size::Four,
            })
            .can_declare(context.hand)
        {
            return Err(refused(DeclarationRefusal::ReplacedByFour));
        }
        Ok(Self(declaration))
    }
}

/// Why a declaration can't be made.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
pub enum DeclarationRefusal {
    /// The rules don't allow declarations on this trick.
    #[error("declarations aren't allowed on this trick")]
    WindowClosed,
    /// The hand doesn't hold its cards.
    #[error("the hand doesn't hold its cards")]
    NotHeld,
    /// It, or an accuso of the same rank, was already declared.
    #[error("it was already declared")]
    AlreadyDeclared,
    /// The hand holds all four cards of the rank, so only four of a kind may
    /// be declared.
    #[error("the hand holds all four, so only four of a kind may be declared")]
    ReplacedByFour,
}

/// Returns the declarations a seat holding `hand` may still make, having
/// already made `declared`.
///
/// A napoli may share its cards with an accuso, but a seat makes at most one
/// accuso per rank, and four of a kind replaces three: holding all four cards
/// of a rank, only the four may be declared.
pub fn allowed(hand: &[TressetteCard], declared: &[Declaration]) -> Vec<Declaration> {
    Declaration::all()
        .filter(|declaration| declaration.can_declare(hand))
        .filter(|declaration| match *declaration {
            Declaration::Napoli(_) => !declared.contains(declaration),
            Declaration::Accuso { rank, size } => {
                let replaced_by_four = size == Size::Three
                    && Declaration::Accuso {
                        rank,
                        size: Size::Four,
                    }
                    .can_declare(hand);
                let rank_taken = declared.iter().any(
                    |made| matches!(made, Declaration::Accuso { rank: taken, .. } if *taken == rank),
                );
                !replaced_by_four && !rank_taken
            }
        })
        .collect()
}

/// The turns of a hand on which a declaration may be made.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Window {
    /// Only on the first trick.
    FirstTrick,
    /// On any of the first three tricks.
    #[default]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Visibility {
    /// Everyone learns that a declaration was made and its points, but not
    /// which one.
    Anonymous,
    /// Everyone learns the exact declaration.
    #[default]
    Exact,
}

/// The declaration rules of one regional variant. The default allows
/// declarations on the first three tricks, with full visibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Rules {
    /// The tricks on which a declaration may be made.
    pub window: Window,
    /// What other seats learn from a declaration.
    pub visibility: Visibility,
}

#[cfg(test)]
/// Test utilities for the declaration module.
pub mod test_utils {
    use super::{DeclarableRank, Declaration, Rules, Visibility, Window};
    use crate::tressette::PerSeat;
    use proptest::{prelude::*, sample};

    /// Generates any rank that can be declared as a set.
    pub fn declarable_rank_strategy() -> impl Strategy<Value = DeclarableRank> {
        sample::select(DeclarableRank::ALL.to_vec())
    }

    /// Generates any declaration, whether or not it could be made.
    pub fn declaration_strategy() -> impl Strategy<Value = Declaration> {
        sample::select(Declaration::all().collect::<Vec<_>>())
    }

    /// Generates any set of distinct declarations, whether or not they could
    /// be made.
    pub fn declarations_strategy() -> impl Strategy<Value = Vec<Declaration>> {
        let all: Vec<_> = Declaration::all().collect();
        let size = all.len();
        sample::subsequence(all, 0..=size)
    }

    /// Generates any declarations for each seat, whether or not they could be
    /// made.
    pub fn declared_by_seat_strategy() -> impl Strategy<Value = PerSeat<Vec<Declaration>>> {
        (
            declarations_strategy(),
            declarations_strategy(),
            declarations_strategy(),
            declarations_strategy(),
        )
            .prop_map(|(north, east, south, west)| PerSeat {
                north,
                east,
                south,
                west,
            })
    }

    /// Generates any declaration rules.
    pub fn rules_strategy() -> impl Strategy<Value = Rules> {
        let window = sample::select(vec![Window::FirstTrick, Window::FirstThreeTricks]);
        let visibility = sample::select(vec![Visibility::Anonymous, Visibility::Exact]);
        (window, visibility).prop_map(|(window, visibility)| Rules { window, visibility })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        test_utils::{
            declarable_rank_strategy, declaration_strategy, declarations_strategy, rules_strategy,
        },
        *,
    };
    use crate::cards::italian::test_utils::suit_strategy;
    use crate::tressette::{
        HAND_SIZE,
        card::test_utils::{cards_strategy, tressette_card_strategy},
    };
    use proptest::prelude::*;

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
    fn cards_of_rank(rank: DeclarableRank, count: usize) -> Vec<TressetteCard> {
        Suit::ALL
            .iter()
            .take(count)
            .map(|&suit| TressetteCard::new(rank.into(), suit))
            .collect()
    }

    fn copies_of(hand: &[TressetteCard], rank: DeclarableRank) -> usize {
        hand.iter()
            .filter(|card| card.rank() == ItalianRank::from(rank))
            .count()
    }

    /// Returns whether `declared` holds an accuso of `rank`, of either size.
    fn accuso_declared(declared: &[Declaration], rank: DeclarableRank) -> bool {
        declared
            .iter()
            .any(|declaration| matches!(declaration, Declaration::Accuso { rank: declared, .. } if *declared == rank))
    }

    proptest! {
        /// Every napoli, and both sizes of accuso of every declarable rank,
        /// is among all the declarations.
        #[test]
        fn all_declarations_cover_every_napoli_and_accuso(
            suit in suit_strategy(),
            rank in declarable_rank_strategy(),
        ) {
            let all: Vec<_> = Declaration::all().collect();
            let three = Declaration::Accuso { rank, size: Size::Three };
            let four = Declaration::Accuso { rank, size: Size::Four };
            prop_assert!(all.contains(&Declaration::Napoli(suit)));
            prop_assert!(all.contains(&three));
            prop_assert!(all.contains(&four));
        }

        /// A napoli is allowed exactly when the hand holds it and it hasn't
        /// been declared, whatever accusi were declared.
        #[test]
        fn a_napoli_is_allowed_when_held_and_not_yet_declared(
            suit in suit_strategy(),
            held in any::<bool>(),
            hand in cards_strategy(),
            declared in declarations_strategy(),
        ) {
            let hand = if held { with_cards(&hand, &napoli_cards(suit)) } else { hand };
            let napoli = Declaration::Napoli(suit);
            let expected = napoli.can_declare(&hand) && !declared.contains(&napoli);
            prop_assert_eq!(allowed(&hand, &declared).contains(&napoli), expected);
        }

        /// Four of a kind is allowed exactly when the hand holds all four
        /// cards of the rank and no accuso of it was declared.
        #[test]
        fn four_of_a_kind_is_allowed_when_held_and_the_rank_is_free(
            rank in declarable_rank_strategy(),
            copies in 0..=Size::Four as usize,
            hand in cards_strategy(),
            declared in declarations_strategy(),
        ) {
            let hand = with_cards(&hand, &cards_of_rank(rank, copies));
            let four = Declaration::Accuso { rank, size: Size::Four };
            let expected = copies_of(&hand, rank) == Size::Four as usize
                && !accuso_declared(&declared, rank);
            prop_assert_eq!(allowed(&hand, &declared).contains(&four), expected);
        }

        /// Three of a kind is allowed exactly when the hand holds three cards
        /// of the rank, not all four, and no accuso of it was declared.
        #[test]
        fn three_of_a_kind_is_allowed_when_four_is_not_held_and_the_rank_is_free(
            rank in declarable_rank_strategy(),
            copies in 0..=Size::Four as usize,
            hand in cards_strategy(),
            declared in declarations_strategy(),
        ) {
            let hand = with_cards(&hand, &cards_of_rank(rank, copies));
            let three = Declaration::Accuso { rank, size: Size::Three };
            let expected = copies_of(&hand, rank) == Size::Three as usize
                && !accuso_declared(&declared, rank);
            prop_assert_eq!(allowed(&hand, &declared).contains(&three), expected);
        }

        /// Any hand that holds a set can declare it, whatever else it holds.
        #[test]
        fn a_hand_holding_the_set_can_declare_it(
            suit in suit_strategy(),
            rank in declarable_rank_strategy(),
            hand in cards_strategy(),
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
            hand in cards_strategy(),
        ) {
            let held = napoli_cards(suit).iter().all(|card| hand.contains(card));
            prop_assert_eq!(Declaration::Napoli(suit).can_declare(&hand), held);
        }

        /// Three of a kind is declarable exactly when the hand holds at least
        /// three cards of its rank.
        #[test]
        fn three_of_a_kind_needs_three_cards_of_its_rank(
            rank in declarable_rank_strategy(),
            hand in cards_strategy(),
        ) {
            let expected = copies_of(&hand, rank) >= Size::Three as usize;
            let declaration = Declaration::Accuso { rank, size: Size::Three };
            prop_assert_eq!(declaration.can_declare(&hand), expected);
        }

        /// Four of a kind is declarable exactly when the hand holds all four
        /// cards of its rank.
        #[test]
        fn four_of_a_kind_needs_four_cards_of_its_rank(
            rank in declarable_rank_strategy(),
            hand in cards_strategy(),
        ) {
            let expected = copies_of(&hand, rank) >= Size::Four as usize;
            let declaration = Declaration::Accuso { rank, size: Size::Four };
            prop_assert_eq!(declaration.can_declare(&hand), expected);
        }

        /// Adding a card to a hand that holds a set never removes the ability
        /// to declare it.
        #[test]
        fn adding_a_card_never_removes_a_declaration(
            suit in suit_strategy(),
            rank in declarable_rank_strategy(),
            hand in cards_strategy(),
            extra in tressette_card_strategy(),
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

        /// Accusi are made of the three strongest ranks of a suit, from the
        /// strongest down.
        #[test]
        fn declarable_ranks_are_the_three_strongest(suit in suit_strategy()) {
            let mut by_strength: Vec<_> = ItalianRank::ALL
                .iter()
                .map(|&rank| TressetteCard::new(rank, suit))
                .collect();
            by_strength.sort_by(|a, b| b.cmp(a));
            let declarable: Vec<_> = DeclarableRank::ALL
                .iter()
                .map(|&rank| TressetteCard::new(rank.into(), suit))
                .collect();
            prop_assert_eq!(declarable, by_strength[..Size::Three as usize].to_vec());
        }

        /// The points depend only on the kind of declaration.
        #[test]
        fn points_depend_only_on_the_kind(suit in suit_strategy(), rank in declarable_rank_strategy()) {
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

        /// Outside the tricks the rules allow, no declaration can be made,
        /// whatever the hand holds.
        #[test]
        fn no_declaration_is_made_outside_the_window(
            rules in rules_strategy(),
            trick in 1..=HAND_SIZE,
            hand in cards_strategy(),
            declaration in declaration_strategy(),
            declared in declarations_strategy(),
        ) {
            prop_assume!(!rules.window.allows(trick));
            let declared: Vec<_> = declared.into_iter().map(Declared).collect();
            let context = Context { rules, trick, hand: &hand, declared: &declared };
            prop_assert_eq!(
                Declared::parse(declaration, &context).map(|_| ()),
                Err(Error::CannotDeclare { declaration, reason: DeclarationRefusal::WindowClosed })
            );
        }

        /// On the first trick, which every window allows, a declaration can be
        /// made exactly when the hand holds it, neither it nor an accuso of its
        /// rank was declared, and it isn't three of a kind while holding all
        /// four; a refusal names the first of these that fails.
        #[test]
        fn a_declaration_is_made_exactly_when_held_and_free(
            rules in rules_strategy(),
            hand in cards_strategy(),
            declaration in declaration_strategy(),
            declared in declarations_strategy(),
        ) {
            let trick = 1;
            let refused = |reason| Err(Error::CannotDeclare { declaration, reason });
            let already = match declaration {
                Declaration::Napoli(_) => declared.contains(&declaration),
                Declaration::Accuso { rank, .. } => accuso_declared(&declared, rank),
            };
            let replaced_by_four = matches!(
                declaration,
                Declaration::Accuso { rank, size: Size::Three } if copies_of(&hand, rank) == Size::Four as usize
            );
            let expected = if !declaration.can_declare(&hand) {
                refused(DeclarationRefusal::NotHeld)
            } else if already {
                refused(DeclarationRefusal::AlreadyDeclared)
            } else if replaced_by_four {
                refused(DeclarationRefusal::ReplacedByFour)
            } else {
                Ok(())
            };
            let declared: Vec<_> = declared.into_iter().map(Declared).collect();
            let context = Context { rules, trick, hand: &hand, declared: &declared };
            prop_assert_eq!(Declared::parse(declaration, &context).map(|_| ()), expected);
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
