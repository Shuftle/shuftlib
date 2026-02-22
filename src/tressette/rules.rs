use crate::core::{
    Suit,
    deck::Deck,
    trick_taking::{Hand, OngoingTrick, PLAYERS, Player, PlayerId, TrickTakingGame},
};
use num_rational::Rational32;

use super::card::TressetteCard;
use anyhow::anyhow;

/// Errors that can occur in Tressette game rule validation.
#[derive(Debug, thiserror::Error)]
pub enum RulesError {
    /// The specified card index is out of bounds for the player's hand.
    #[error("Card index {index} is out of bounds (hand has {hand_size} cards)")]
    CardIndexOutOfBounds {
        /// The invalid index that was requested
        index: usize,
        /// The actual number of cards in the player's hand
        hand_size: usize,
    },
    /// The card cannot be played because it doesn't follow the required suit.
    #[error("Must follow {required_suit} suit when led (played {played_card})")]
    MustFollowSuit {
        /// The suit that must be followed
        required_suit: Suit,
        /// The card that was attempted to be played
        played_card: TressetteCard,
    },
    /// An internal error occurred.
    #[error(transparent)]
    Internal(anyhow::Error),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
/// Contains the rules of the tressette game.
pub struct TressetteRules {}

impl TrickTakingGame for TressetteRules {
    type CardType = TressetteCard;

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
    /// use shuftlib::trick_taking::{TrickTakingGame, PlayerId};
    /// use shuftlib::core::Suit;
    /// use shuftlib::core::italian::ItalianRank;
    /// use shuftlib::tressette::{TressetteRules, TressetteCard};
    ///
    /// let cards = [
    ///   TressetteCard::new(ItalianRank::Ace, Suit::Hearts),
    ///   TressetteCard::new(ItalianRank::Two, Suit::Hearts),
    ///   TressetteCard::new(ItalianRank::Three, Suit::Hearts),
    ///   TressetteCard::new(ItalianRank::Four, Suit::Hearts),
    /// ];
    ///
    /// let taker = TressetteRules::determine_taker(&cards, PlayerId::PLAYER_2);
    /// assert_eq!(taker, PlayerId::PLAYER_2);
    /// ```
    #[allow(clippy::expect_used)]
    fn determine_taker(cards: &[TressetteCard; PLAYERS], first_to_play: PlayerId) -> PlayerId {
        let leading_suit = cards[first_to_play.as_usize()].suit();
        let (taker, _) = cards
            .iter()
            .enumerate()
            .filter(|&(_, &c)| c.suit() == leading_suit)
            .max_by_key(|&(_, &c)| c)
            .expect("Max by key returned None. This shouldn't have happened, since it's being called on a non empty slice.");

        PlayerId::try_from(taker).expect("Initialization of a new PlayerId failed. This shouldn't have happened, since the input usize was computed starting from a fixed length slice.")
    }

    fn is_game_over(scores: (u8, u8)) -> bool {
        (scores.0 >= SCORE_TO_WIN && scores.0 > scores.1)
            || (scores.1 >= SCORE_TO_WIN && scores.1 > scores.0)
    }

    fn score_hand(hand: &Hand<Self>) -> (u8, u8) {
        let mut tmp_score = (Rational32::new(0, 3), Rational32::new(0, 3));

        let mut taker = 0;
        for trick in hand.tricks() {
            taker = trick.taker().as_usize();
            if taker == 0 || taker == 2 {
                tmp_score.0 += trick.cards().iter().map(|c| c.value()).sum::<Rational32>();
            } else {
                tmp_score.1 += trick.cards().iter().map(|c| c.value()).sum::<Rational32>();
            }
        }

        let mut score = (
            tmp_score.0.to_integer() as u8,
            tmp_score.1.to_integer() as u8,
        );

        // The game awards 1 extra point to the taker of the last trick.
        if taker == 0 || taker == 2 {
            score.0 += 1;
        } else {
            score.1 += 1;
        }

        score
    }

    fn deck() -> Deck<Self::CardType> {
        Deck::italian()
            .into_iter()
            .map(TressetteCard::from)
            .collect()
    }
}

/// The score a team has to reach to win a game of tressette.
pub const SCORE_TO_WIN: u8 = 31;

impl TressetteRules {
    /// Number of cards dealt to each player at the start of a Tressette hand.
    pub const HAND_SIZE: usize = 10;

    /// Returns playable cards along with their indices in the player's hand.
    ///
    /// This is useful for index-based card selection in UIs where users click
    /// on cards at specific positions.
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::tressette::{TressetteRules, TressetteCard};
    /// use shuftlib::trick_taking::Player;
    /// use shuftlib::core::Suit;
    /// use shuftlib::core::italian::ItalianRank;
    ///
    /// let mut player = Player::default();
    /// player.give(TressetteCard::new(ItalianRank::Ace, Suit::Spades));
    /// player.give(TressetteCard::new(ItalianRank::Two, Suit::Hearts));
    ///
    /// let playable = TressetteRules::playable(&player, None);
    /// assert_eq!(playable.len(), 2);
    /// assert_eq!(playable[0].0, 0); // index
    /// assert_eq!(playable[1].0, 1); // index
    /// ```
    pub fn playable(
        player: &Player<TressetteRules>,
        leading_suit: Option<Suit>,
    ) -> Vec<(usize, TressetteCard)> {
        if let Some(leading_suit) = leading_suit
            && player.hand().iter().any(|c| c.suit() == leading_suit)
        {
            return player
                .hand()
                .iter()
                .enumerate()
                .filter(|(_, c)| c.suit() == leading_suit)
                .map(|(i, &card)| (i, card))
                .collect();
        }

        player
            .hand()
            .iter()
            .enumerate()
            .map(|(i, &card)| (i, card))
            .collect()
    }

    /// Plays the card at the specified index for the player.
    ///
    /// This method validates that:
    /// 1. The index is within bounds of the player's hand
    /// 2. The card at that index is actually playable given the game rules
    ///
    /// This provides protection against cheating or UI bugs where an invalid
    /// card might be selected.
    ///
    /// # Errors
    ///
    /// Returns an error if the index is out of bounds or if the card at that
    /// index is not playable according to the current game rules.
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::tressette::{TressetteRules, TressetteCard};
    /// use shuftlib::trick_taking::{Player, PlayerId, OngoingTrick};
    /// use shuftlib::core::Suit;
    /// use shuftlib::core::italian::ItalianRank;
    ///
    /// let mut player = Player::default();
    /// player.give(TressetteCard::new(ItalianRank::Ace, Suit::Spades));
    /// let mut trick = OngoingTrick::<TressetteRules>::new(PlayerId::PLAYER_0);
    ///
    /// // Playing a valid index succeeds
    /// let result = TressetteRules::play(&mut player, 0, &mut trick, None);
    /// assert!(result.is_ok());
    /// ```
    pub fn play(
        player: &mut Player<TressetteRules>,
        index: usize,
        ongoing_trick: &mut OngoingTrick<TressetteRules>,
        leading_suit: Option<Suit>,
    ) -> Result<TressetteCard, RulesError> {
        let card = player
            .hand()
            .get(index)
            .ok_or(RulesError::CardIndexOutOfBounds {
                index,
                hand_size: player.hand().len(),
            })?;

        // Validate the card is actually playable
        let is_playable = if let Some(leading_suit) = leading_suit {
            // If there's a leading suit, check if player has any cards of that suit
            let has_leading_suit = player.hand().iter().any(|c| c.suit() == leading_suit);
            if has_leading_suit {
                // Must play the leading suit
                card.suit() == leading_suit
            } else {
                // No cards of leading suit, so any card is playable
                true
            }
        } else {
            // No leading suit, any card is playable
            true
        };

        if !is_playable {
            let required_suit = leading_suit.ok_or_else(|| {
                RulesError::Internal(anyhow!("Leading suit should be Some if not playable"))
            })?;
            return Err(RulesError::MustFollowSuit {
                required_suit,
                played_card: *card,
            });
        }

        let card = player.remove_card(index).ok_or_else(|| {
            RulesError::Internal(anyhow!("Card removal should succeed after validation"))
        })?;
        ongoing_trick.play(card);
        Ok(card)
    }
}

#[cfg(test)]
mod test_utils {
    use crate::core::{
        Suit,
        italian::ItalianRank,
        trick_taking::{Player, PlayerId},
    };
    use proptest::collection::hash_set;
    use proptest::prelude::*;

    use super::TressetteRules;
    use crate::tressette::TressetteCard;

    /// Strategy to create a random `TressetteCard`.
    pub fn tressette_card_strategy() -> impl Strategy<Value = TressetteCard> {
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

    /// Strategy to create a `Player<TressetteRules>` with random cards.
    pub fn player_strategy() -> impl Strategy<Value = Player<TressetteRules>> {
        (
            0..crate::trick_taking::PLAYERS,
            hash_set(tressette_card_strategy(), 1..crate::trick_taking::TRICKS),
        )
            .prop_map(|(index, cards)| {
                let player_id = PlayerId::try_from(index).ok()?;
                let mut player = Player::new(player_id);
                for card in cards {
                    player.give(card);
                }
                Some(player)
            })
            .prop_filter_map("valid player id", |opt| opt)
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use crate::trick_taking::PlayerId;
    use crate::{core::Suit, trick_taking::TrickTakingGame};

    use super::{SCORE_TO_WIN, TressetteRules};

    proptest! {
        #[test]
        fn a_team_won_with_both_below(team1_score in 0u8..SCORE_TO_WIN, team2_score in 0u8..SCORE_TO_WIN) {
            let result = TressetteRules::is_game_over((team1_score, team2_score));
            assert!(!result);
        }

        #[test]
        fn a_team_won_with_both_above_and_same(score in SCORE_TO_WIN..u8::MAX) {
            let result = TressetteRules::is_game_over((score, score));
            assert!(!result);
        }

        #[test]
        fn a_team_won_with_both_above_and_different(score in SCORE_TO_WIN..u8::MAX) {
            let result = TressetteRules::is_game_over((score, score + 1));
            assert!(result);
        }

        #[test]
        fn a_team_won_with_team1_above(team1_score in 0u8..SCORE_TO_WIN, team2_score in SCORE_TO_WIN..u8::MAX) {
            let result = TressetteRules::is_game_over((team1_score, team2_score));
            assert!(result);
        }

        #[test]
        fn a_team_won_with_team2_above(team1_score in SCORE_TO_WIN..u8::MAX, team2_score in 0u8..SCORE_TO_WIN ) {
            let result = TressetteRules::is_game_over((team1_score, team2_score));
            assert!(result);
        }



        #[test]
        fn playable_returns_correct_indices(player in super::test_utils::player_strategy(), suit in prop_oneof![Just(Suit::Hearts), Just(Suit::Spades), Just(Suit::Clubs), Just(Suit::Diamonds)]) {
            let playable = TressetteRules::playable(&player, None);

            // All indices should be valid
            for (index, card) in &playable {
                prop_assert!(*index < player.hand().len());
                prop_assert_eq!(player.hand()[*index], *card);
            }

            // When no leading suit, all cards should be playable
            prop_assert_eq!(playable.len(), player.hand().len());

            // Test with leading suit
            let playable_suited = TressetteRules::playable(&player, Some(suit));
            let same_suit_cards = player.hand().iter().filter(|c| c.suit() == suit).count();

            // All returned cards should be of the leading suit (if any exist)
            if same_suit_cards > 0 {
                for (index, card) in &playable_suited {
                    prop_assert_eq!(card.suit(), suit);
                    prop_assert_eq!(player.hand()[*index], *card);
                }
                prop_assert_eq!(playable_suited.len(), same_suit_cards);
            } else {
                // If no cards of leading suit, all cards are playable
                prop_assert_eq!(playable_suited.len(), player.hand().len());
            }
        }

        #[test]
        fn remove_card_at_valid_index_removes_correct_card(player in super::test_utils::player_strategy(), index in 0usize..10) {
            let mut player = player;
            let original_len = player.hand().len();

            if index < original_len {
                let expected_card = player.hand()[index];
                let removed = player.remove_card(index);

                prop_assert_eq!(removed, Some(expected_card));
                prop_assert_eq!(player.hand().len(), original_len - 1);
                // Card should no longer be in hand at that index
                if index < player.hand().len() {
                    prop_assert_ne!(player.hand()[index], expected_card);
                }
            } else {
                // Invalid index should not modify hand
                let removed = player.remove_card(index);
                prop_assert_eq!(removed, None);
                prop_assert_eq!(player.hand().len(), original_len);
            }
        }

        #[test]
        fn play_validates_playability(player in super::test_utils::player_strategy(), index in 0usize..10, suit in prop_oneof![Just(Suit::Hearts), Just(Suit::Spades), Just(Suit::Clubs), Just(Suit::Diamonds)]) {
            let mut player = player;
            let original_hand_len = player.hand().len();
            let mut trick = crate::trick_taking::OngoingTrick::<TressetteRules>::new(PlayerId::PLAYER_0);

            // Get playable indices
            let playable = TressetteRules::playable(&player, Some(suit));
            let playable_indices: Vec<usize> = playable.iter().map(|(i, _)| *i).collect();

            let result = TressetteRules::play(&mut player, index, &mut trick, Some(suit));

            if index < original_hand_len && playable_indices.contains(&index) {
                // Should succeed - valid and playable
                prop_assert!(result.is_ok());
                prop_assert_eq!(player.hand().len(), original_hand_len - 1);
            } else {
                // Should fail - either out of bounds or not playable
                prop_assert!(result.is_err());
                prop_assert_eq!(player.hand().len(), original_hand_len);
            }
        }

        #[test]
        fn play_with_no_suit_constraint_allows_any_card(player in super::test_utils::player_strategy(), index in 0usize..10) {
            let mut player = player;
            let original_hand_len = player.hand().len();
            let mut trick = crate::trick_taking::OngoingTrick::<TressetteRules>::new(PlayerId::PLAYER_0);

            let result = TressetteRules::play(&mut player, index, &mut trick, None);

            if index < original_hand_len {
                // Should succeed - any card is playable when no suit constraint
                prop_assert!(result.is_ok());
                prop_assert_eq!(player.hand().len(), original_hand_len - 1);
            } else {
                // Should fail - out of bounds
                prop_assert!(result.is_err());
                prop_assert_eq!(player.hand().len(), original_hand_len);
            }
        }
    }
}
