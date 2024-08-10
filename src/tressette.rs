use std::{fmt::Display, ops::Deref};

use crate::common::{
    cards::{Card, ItalianCard, ItalianRank, Suit},
    hands::{Hand, OngoingTrick, Player, PlayerId, TrickTakingGame},
};
use num_rational::Rational32;
use std::cmp::Ordering;

#[derive(Clone, Debug, Default)]
/// Contains the rules of the tressette game.
pub struct TressetteRules {}

impl TrickTakingGame for TressetteRules {
    type CardType = TressetteCard;

    const PLAYERS: usize = 4;
    const TRICKS: usize = 10;

    /// Contains the logic to determine who won the trick in a standard
    /// tressette game: The winner of the trick is always the player who played
    /// the highest card with the same `Suit` of the first `TressetteCard`
    /// played that trick. See the implementation of `Ord` and `PartialOrd` for
    /// `TressetteCard` for more info. The implementation of this trait is meant
    /// to only be used internally by `OngoingTrick`, however it's possible to
    /// call it elsewhere if needed. It also assumes the slice `cards` is valid
    /// for the tressette game, so it assumes there are no duplicates. It's a
    /// responsability of the caller to make sure that's the case.
    ///
    /// # Panics
    ///
    /// It can only panic in case of a bug in this crate.
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::common::{hands::{TrickTakingGame, PlayerId}, cards::{ItalianRank, Suit}};
    /// use shuftlib::tressette::{TressetteRules, TressetteCard};
    ///
    /// let cards = [
    ///   TressetteCard::new(ItalianRank::Ace, Suit::Hearts),
    ///   TressetteCard::new(ItalianRank::Two, Suit::Hearts),
    ///   TressetteCard::new(ItalianRank::Three, Suit::Hearts),
    ///   TressetteCard::new(ItalianRank::Four, Suit::Hearts),
    /// ];
    ///
    /// let taker = TressetteRules::determine_taker(&cards, PlayerId::new(2).unwrap());
    /// assert_eq!(taker, PlayerId::new(2).unwrap());
    /// ```
    #[allow(clippy::expect_used)]
    fn determine_taker(
        cards: &[TressetteCard; Self::PLAYERS],
        first_to_play: PlayerId<{ Self::PLAYERS }>,
    ) -> PlayerId<{ Self::PLAYERS }> {
        let leading_suit = cards[*first_to_play].suit();
        let (taker, _) = cards
            .iter()
            .enumerate()
            .filter(|(_, &c)| c.suit() == leading_suit)
            .max_by_key(|(_, &c)| c)
            .expect("Max by key returned None. This shouldn't have happened, since it's being called on a non empty slice.");

        PlayerId::new(taker).expect("Initialization of a new PlayerId failed. This shouldn't have happened, since the input usize was computed starting from a fixed length slice.")
    }
}

/// The score a team has to reach to win a game of tressette.
pub const SCORE_TO_WIN: u8 = 31;

impl TressetteRules {
    /// Determines if a team won the game. A team wins the game when its score is
    /// greater than 31 and has a higher score than the other team.
    pub fn is_completed(score: (u8, u8)) -> bool {
        (score.0 >= SCORE_TO_WIN && score.0 > score.1)
            || (score.1 >= SCORE_TO_WIN && score.1 > score.0)
    }

    /// Returns a view of the playable cards held by a player, based on the suit
    /// of a card that has been played before and by the rules of tressette. If
    /// the player is the first to play, the leading suit can be None.
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::tressette::{TressetteRules, TressetteCard};
    /// use shuftlib::common::hands::Player;
    /// use shuftlib::common::cards::{Suit, ItalianRank};
    ///
    /// let mut player = Player::default();
    /// player.give(TressetteCard::new(ItalianRank::Ace, Suit::Spades));
    /// player.give(TressetteCard::new(ItalianRank::Two, Suit::Spades));
    /// player.give(TressetteCard::new(ItalianRank::Ace, Suit::Hearts));
    ///
    /// assert_eq![TressetteRules::playable(&player, Some(Suit::Spades)).len(), 2];
    /// assert_eq![TressetteRules::playable(&player, Some(Suit::Clubs)).len(), 3];
    /// ```
    pub fn playable(
        player: &Player<TressetteRules>,
        leading_suit: Option<Suit>,
    ) -> Vec<TressetteCard> {
        if let Some(leading_suit) = leading_suit {
            if player.hand().iter().any(|c| c.suit() == leading_suit) {
                return player
                    .hand()
                    .iter()
                    .filter(|c| c.suit() == leading_suit)
                    .cloned()
                    .collect();
            }
        }

        player.hand().into()
    }

    /// Plays the specified card for the player
    pub fn play(
        player: &mut Player<TressetteRules>,
        card: TressetteCard,
        ongoing_trick: &mut OngoingTrick<TressetteRules>,
    ) {
        player.remove(card);
        ongoing_trick.play(card);
    }

    /// Computes the score for a hand of the tressette game.
    /// Score is always a maximum of 11 points.
    pub fn compute_score(hand: &Hand<Self>, score: &mut (u8, u8)) {
        let mut tmp_score = (Rational32::new(0, 3), Rational32::new(0, 3));

        let mut taker = 0;
        for trick in hand.tricks() {
            if *trick.taker() == 0 || *trick.taker() == 2 {
                tmp_score.0 += trick.cards().iter().map(|c| c.value()).sum::<Rational32>();
            } else {
                tmp_score.1 += trick.cards().iter().map(|c| c.value()).sum::<Rational32>();
            }

            taker = *trick.taker();
        }

        score.0 += tmp_score.0.to_integer() as u8;
        score.1 += tmp_score.1.to_integer() as u8;

        if taker == 0 || taker == 2 {
            score.0 += 1;
        } else {
            score.1 += 1;
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Default, Hash)]
/// Representation of a card used in variations of the Tressette game. It's just
/// a new type over `ItalianCard`.
pub struct TressetteCard {
    card: ItalianCard,
}

impl PartialOrd for TressetteCard {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TressetteCard {
    #[allow(clippy::expect_used)]
    fn cmp(&self, other: &Self) -> Ordering {
        let rank_order = [
            ItalianRank::Four,
            ItalianRank::Five,
            ItalianRank::Six,
            ItalianRank::Seven,
            ItalianRank::Jack,
            ItalianRank::Knight,
            ItalianRank::King,
            ItalianRank::Ace,
            ItalianRank::Two,
            ItalianRank::Three,
        ];

        let self_rank_index = rank_order.iter().position(|&r| self.card.rank() == r).expect("The rank of self wasn't found inside the Ord implementation for TressetteCard. This shouldn't have happened, please file a bug report.");
        let other_rank_index = rank_order.iter().position(|&r| other.card.rank() == r).expect("The rank of other wasn't found inside the Ord implementation for TressetteCard. This shouldn't have happened, please file a bug report.");

        self_rank_index.cmp(&other_rank_index)
    }
}

impl Display for TressetteCard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.card)
    }
}

impl Card for TressetteCard {}

impl From<ItalianCard> for TressetteCard {
    fn from(value: ItalianCard) -> Self {
        TressetteCard { card: value }
    }
}

impl Deref for TressetteCard {
    type Target = ItalianCard;

    fn deref(&self) -> &Self::Target {
        &self.card
    }
}

impl TressetteCard {
    /// Gets the value of the card by the rules of the Tressette game:
    /// - Ace = 1
    /// - 2, 3 and figures = 1/3
    /// - the rest = 0/3
    ///
    /// # Examples
    /// ```
    /// use shuftlib::{tressette::TressetteCard, common::cards::{Suit, ItalianRank}};
    /// use num_rational::Rational32;
    ///
    /// let ace = TressetteCard::new(ItalianRank::Ace, Suit::Hearts);
    /// let two = TressetteCard::new(ItalianRank::Two, Suit::Spades);
    /// let four = TressetteCard::new(ItalianRank::Four, Suit::Clubs);
    /// assert_eq!(ace.value(), Rational32::new(3,3));
    /// assert_eq!(two.value(), Rational32::new(1,3));
    /// assert_eq!(four.value(), Rational32::new(0,3));
    /// ```
    pub fn value(&self) -> Rational32 {
        match self.rank() {
            ItalianRank::Ace => Rational32::new(3, 3),
            ItalianRank::Two
            | ItalianRank::Three
            | ItalianRank::King
            | ItalianRank::Knight
            | ItalianRank::Jack => Rational32::new(1, 3),
            ItalianRank::Four | ItalianRank::Five | ItalianRank::Six | ItalianRank::Seven => {
                Rational32::new(0, 3)
            }
        }
    }

    /// Generates a new `TressetteCard` starting from an `ItalianRank` and
    /// a `Suit`.
    ///
    /// # Examples.
    /// ```
    /// use shuftlib::common::cards::{ItalianCard, ItalianRank, Suit};
    /// use shuftlib::tressette::TressetteCard;
    ///
    /// let suit = Suit::Spades;
    /// let rank = ItalianRank::Ace;
    /// assert_eq!(*TressetteCard::new(rank, suit), ItalianCard::new(rank,suit));
    /// ```
    pub fn new(rank: ItalianRank, suit: Suit) -> Self {
        let card = ItalianCard::new(rank, suit);

        TressetteCard { card }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        common::{
            cards::{ItalianRank, Suit},
            hands::{Player, PlayerId, TrickTakingGame},
        },
        tressette::SCORE_TO_WIN,
    };
    use prop::collection::hash_set;
    use proptest::prelude::*;

    use super::{TressetteCard, TressetteRules};

    fn tressette_card_strategy() -> impl Strategy<Value = TressetteCard> {
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
            .prop_map(|(rank, suit)| TressetteCard::new(rank, suit))
    }

    fn player_strategy() -> impl Strategy<Value = Player<TressetteRules>> {
        (
            0..TressetteRules::PLAYERS,
            hash_set(tressette_card_strategy(), 1..TressetteRules::TRICKS),
        )
            .prop_map(|(index, cards)| {
                let mut player = Player::new(PlayerId::new(index).unwrap());
                for card in cards {
                    player.give(card);
                }
                player
            })
    }

    proptest! {
        #[test]
        fn a_team_won_with_both_below(team1_score in 0u8..SCORE_TO_WIN, team2_score in 0u8..SCORE_TO_WIN) {
            let result = TressetteRules::is_completed((team1_score, team2_score));
            assert!(!result);
        }

        #[test]
        fn a_team_won_with_both_above_and_same(score in SCORE_TO_WIN..u8::MAX) {
            let result = TressetteRules::is_completed((score, score));
            assert!(!result);
        }

        #[test]
        fn a_team_won_with_both_above_and_different(score in SCORE_TO_WIN..u8::MAX) {
            let result = TressetteRules::is_completed((score, score+1));
            assert!(result);
        }

        #[test]
        fn a_team_won_with_team1_above(team1_score in 0u8..SCORE_TO_WIN, team2_score in SCORE_TO_WIN..u8::MAX) {
            let result = TressetteRules::is_completed((team1_score, team2_score));
            assert!(result);
        }

        #[test]
        fn a_team_won_with_team2_above(team1_score in SCORE_TO_WIN..u8::MAX, team2_score in 0u8..SCORE_TO_WIN ) {
            let result = TressetteRules::is_completed((team1_score, team2_score));
            assert!(result);
        }

        #[test]
        fn playable_works(player in player_strategy(), suit in prop_oneof![Just(Suit::Hearts), Just(Suit::Spades), Just(Suit::Clubs), Just(Suit::Diamonds)]) {
            let playable = TressetteRules::playable(&player, None);

            // When the player is the first to go, he can play whatever he wants.
            prop_assert_eq!(playable.len(), player.hand().len());

            let playable = TressetteRules::playable(&player, Some(suit));
            let same_suit_cards = player.hand().iter().filter(|c| c.suit() == suit).count();

            // When the player is not the first, if he doesn't have cards of the
            // same suit as the first, he can play whatever he wants, otherwise
            // he can only play cards of the same suit.
            if same_suit_cards == 0 {
                prop_assert_eq!(playable.len(), player.hand().len());
            } else {
                prop_assert_eq!(playable.len(), same_suit_cards);
            }
        }
    }
}
