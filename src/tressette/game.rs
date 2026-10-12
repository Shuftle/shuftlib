//! High-level game state management for Tressette.

use rand::rngs::StdRng;

use crate::{
    core::{Suit, deck::Deck},
    trick_taking::{Hand, OngoingHand, OngoingTrick, PLAYERS, Player, PlayerId, TrickTakingGame},
};

use super::{TressetteCard, TressetteRules};

/// The effect of making a move in the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveEffect {
    /// A card was played, but the trick is not yet complete.
    CardPlayed,
    /// A trick was completed, and the winner is now the next to play.
    TrickCompleted {
        /// The player who won the trick.
        winner: PlayerId,
    },
    /// A hand was completed, scores were updated, and a new hand was dealt.
    HandComplete {
        /// The player who won the last trick of the hand.
        trick_winner: PlayerId,
        /// The updated scores after the hand: (team_0_2_score, team_1_3_score).
        score: (u8, u8),
    },
    /// The game is over.
    GameOver {
        /// The player who won the last trick.
        trick_winner: PlayerId,
        /// The final scores: (team_0_2_score, team_1_3_score).
        final_score: (u8, u8),
    },
}

/// The current status of the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The game is ongoing.
    Ongoing,
    /// The game is finished.
    Finished {
        /// The winning team (team_0_2 or team_1_3).
        winner: u8,
    },
}

/// An error that can occur when making a move.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// The card is not in the current player's hand.
    #[error("Card {card} is not in player {player}'s hand")]
    CardNotInHand {
        /// The card that was attempted.
        card: TressetteCard,
        /// The player who tried to play it.
        player: PlayerId,
    },
    /// The player must follow suit but did not.
    #[error("Must follow suit: played {suit} but {required_suit} is required")]
    MustFollowSuit {
        /// Attempted Suit.
        suit: Suit,
        /// The suit that must be followed.
        required_suit: Suit,
    },
    /// The game is already over.
    #[error("Game is already over")]
    GameOver,
    /// An internal state error occurred.
    ///
    /// This indicates a bug in the library.
    #[error("Internal state error: {0}")]
    InternalError(String),
}

/// The state of a Tressette game at a given point.
///
/// This includes the players' hands, the current trick, scores, history, and other game state.
#[derive(Debug)]
pub struct Game {
    deck: Deck<TressetteCard>,
    players: [Player<TressetteRules>; PLAYERS],
    current_hand: OngoingHand<TressetteRules>,
    /// The player who is dealing the current hand.
    dealing_player: PlayerId,
    score: (u8, u8),
    completed_hands: Vec<Hand<TressetteRules>>,
    history: Vec<(TressetteCard, MoveEffect)>,
    /// Seed used for all randomness at construction, enabling deterministic replay.
    seed: u64,
    /// RNG seeded from `seed`, persisted and reused for every reshuffle across
    /// the game's lifetime so replaying from `seed` reproduces it exactly.
    rng: StdRng,
}

impl Game {
    /// Creates a new game with cards dealt.
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::tressette::Game;
    ///
    /// let game = Game::new();
    /// assert_eq!(game.score(), (0, 0));
    /// ```
    pub fn new() -> Self {
        Self::from_seed(rand::random())
    }

    /// Creates a new game from a specific seed, enabling deterministic replay.
    fn from_seed(seed: u64) -> Self {
        use rand::RngExt;
        use rand::SeedableRng;

        let mut rng = StdRng::seed_from_u64(seed);
        let dealing_player = PlayerId::ALL[rng.random_range(0..PLAYERS)];
        let mut deck = TressetteRules::deck();
        deck.shuffle(&mut rng);
        let mut game = Self {
            players: [
                Player::new(PlayerId::PLAYER_0),
                Player::new(PlayerId::PLAYER_1),
                Player::new(PlayerId::PLAYER_2),
                Player::new(PlayerId::PLAYER_3),
            ],
            // Placeholder, immediately overwritten by `initialize_new_hand` below.
            current_hand: OngoingHand::new(dealing_player),
            score: (0, 0),
            completed_hands: Vec::new(),
            history: Vec::new(),
            dealing_player,
            deck,
            seed,
            rng,
        };
        game.initialize_new_hand();
        game.deal();
        game
    }

    /// Returns the player whose turn it is to move.
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::tressette::Game;
    ///
    /// let game = Game::new();
    /// let player = game.current_player();
    /// ```
    pub fn current_player(&self) -> PlayerId {
        self.current_hand.current_trick().next_to_play()
    }

    /// Returns the current scores: (team_0_2_score, team_1_3_score).
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::tressette::Game;
    ///
    /// let game = Game::new();
    /// assert_eq!(game.score(), (0, 0));
    /// ```
    pub fn score(&self) -> (u8, u8) {
        self.score
    }

    /// Returns the current status of the game.
    ///
    /// # Panics
    ///
    /// It can only panic in case of a bug.
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::tressette::{Game, Status};
    ///
    /// let game = Game::new();
    /// assert_eq!(game.status(), Status::Ongoing);
    /// ```
    pub fn status(&self) -> Status {
        if TressetteRules::is_game_over(self.score) {
            let winner = if self.score.0 > self.score.1 {
                0
            } else if self.score.1 > self.score.0 {
                1
            } else {
                unreachable!(
                    "The score of the teams is the same, but the game is completed. This is a bug"
                );
            };
            Status::Finished { winner }
        } else {
            Status::Ongoing
        }
    }

    /// Returns the cards in the given player's hand.
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::{tressette::Game, trick_taking::PlayerId};
    ///
    /// let game = Game::new();
    /// let hand = game.hand(PlayerId::PLAYER_0);
    /// assert_eq!(hand.len(), 10);
    /// ```
    pub fn hand(&self, player: PlayerId) -> &[TressetteCard] {
        self.players[player.as_usize()].hand()
    }

    /// Returns the cards currently played in the ongoing trick.
    ///
    /// The array is indexed by player ID, with `None` for players who haven't played yet.
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::tressette::Game;
    ///
    /// let game = Game::new();
    /// let trick = game.current_trick();
    /// assert!(trick.iter().all(|c| c.is_none()));
    /// ```
    pub fn current_trick(&self) -> &[Option<TressetteCard>; PLAYERS] {
        self.current_hand.current_trick()
    }

    /// Returns the player who led the current trick.
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::{tressette::Game, trick_taking::{PlayerId, PLAYERS}};
    ///
    /// let game = Game::new();
    /// assert!(game.trick_leader().as_usize() < PLAYERS);
    /// ```
    pub fn trick_leader(&self) -> PlayerId {
        self.current_hand.current_trick().first_to_play()
    }

    /// Returns the number of tricks completed in the current hand.
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::tressette::Game;
    ///
    /// let game = Game::new();
    /// assert_eq!(game.tricks_this_hand(), 0);
    /// ```
    pub fn tricks_this_hand(&self) -> usize {
        self.current_hand
            .tricks()
            .iter()
            .filter(|t| t.is_some())
            .count()
    }

    /// Returns the number of hands completed so far.
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::tressette::Game;
    ///
    /// let game = Game::new();
    /// assert_eq!(game.hands_completed(), 0);
    /// ```
    pub fn hands_completed(&self) -> usize {
        self.completed_hands.len()
    }

    /// Returns all legal cards for the current player.
    ///
    /// Returns an empty vector if the game is over.
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::tressette::Game;
    ///
    /// let game = Game::new();
    /// let cards = game.legal_cards();
    /// assert!(!cards.is_empty());
    /// ```
    pub fn legal_cards(&self) -> Vec<TressetteCard> {
        if matches!(self.status(), Status::Finished { .. }) {
            return vec![];
        }

        let hand = self.players[self.current_player().as_usize()].hand();

        let leading_suit = self
            .current_hand
            .current_trick()
            .iter()
            .flatten()
            .next()
            .map(|c| c.suit());

        match leading_suit {
            Some(suit) => {
                let same_suit: Vec<_> = hand
                    .iter()
                    .filter(|&&c| c.suit() == suit)
                    .copied()
                    .collect();
                if same_suit.is_empty() {
                    hand.to_vec()
                } else {
                    same_suit
                }
            }
            None => hand.to_vec(),
        }
    }

    /// Returns `true` if the given card is legal for the current player.
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::{tressette::{Game, TressetteCard}, core::{Suit, italian::ItalianRank}};
    ///
    /// let game = Game::new();
    /// let card = game.hand(game.current_player())[0];
    /// assert!(game.is_legal_card(card));
    /// ```
    pub fn is_legal_card(&self, card: TressetteCard) -> bool {
        self.legal_cards().contains(&card)
    }

    /// Plays a card, updating the game state and recording the card in history.
    ///
    /// This is the core state transition function.
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::tressette::Game;
    ///
    /// let mut game = Game::new();
    /// let legal = game.legal_cards();
    /// let effect = game.play_card(legal[0]).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The game is already over ([`Error::GameOver`]).
    /// - The card is not in the current player's hand ([`Error::CardNotInHand`]).
    /// - The player must follow suit but didn't ([`Error::MustFollowSuit`]).
    /// - Internal state is inconsistent ([`Error::InternalError`]).
    pub fn play_card(&mut self, card: TressetteCard) -> Result<MoveEffect, Error> {
        // Check game not over
        if matches!(self.status(), Status::Finished { .. }) {
            return Err(Error::GameOver);
        }

        // Validate card
        if !self.is_legal_card(card) {
            let hand = self.players[self.current_player().as_usize()].hand();
            if !hand.contains(&card) {
                return Err(Error::CardNotInHand {
                    card,
                    player: self.current_player(),
                });
            } else {
                // Compute required suit
                let required_suit = self
                    .leading_suit()
                    .ok_or(Error::InternalError("No leading suit".to_string()))?;
                return Err(Error::MustFollowSuit {
                    required_suit,
                    suit: card.suit(),
                });
            }
        }

        // Apply move
        let player = &mut self.players[self.current_player().as_usize()];
        let card_idx = player
            .hand()
            .iter()
            .position(|&c| c == card)
            .ok_or_else(|| Error::InternalError("Card should be in hand".to_string()))?;
        player.remove_card(card_idx).ok_or_else(|| {
            Error::InternalError("remove_card failed after index was already validated".to_string())
        })?;

        // Add to trick
        self.current_hand.current_trick_mut().play(card);

        // Check if trick is complete
        let effect = if self.is_trick_complete() {
            self.complete_trick()?
        } else {
            MoveEffect::CardPlayed
        };

        // Record in history
        self.history.push((card, effect));

        Ok(effect)
    }

    /// Returns the suit of the leading card in the current trick, if any.
    fn leading_suit(&self) -> Option<Suit> {
        self.current_hand
            .current_trick()
            .iter()
            .flatten()
            .next()
            .map(|c| c.suit())
    }

    /// Returns the card history.
    ///
    /// Each element is a tuple of the card played and its effect.
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::tressette::Game;
    ///
    /// let mut game = Game::new();
    /// let legal = game.legal_cards();
    /// game.play_card(legal[0]).unwrap();
    /// assert_eq!(game.history().len(), 1);
    /// ```
    pub fn history(&self) -> &[(TressetteCard, MoveEffect)] {
        &self.history
    }

    /// Returns the game state after the first `n` cards in the history.
    ///
    /// Returns `None` if `n` is greater than the number of cards played.
    ///
    /// # Examples
    ///
    /// ```
    /// use shuftlib::tressette::Game;
    ///
    /// let game = Game::new();
    /// let initial_state = game.state_after_card(0);
    /// assert!(initial_state.is_some());
    /// ```
    pub fn state_after_card(&self, n: usize) -> Option<Game> {
        if n > self.history.len() {
            return None;
        }

        let mut game = Self::from_seed(self.seed);
        for (card, _) in &self.history[..n] {
            game.play_card(*card).ok()?;
        }
        Some(game)
    }

    fn is_trick_complete(&self) -> bool {
        self.current_hand
            .current_trick()
            .iter()
            .all(|c| c.is_some())
    }

    fn complete_trick(&mut self) -> Result<MoveEffect, Error> {
        // Finish the current trick
        let trick = (*self.current_hand.current_trick())
            .finish()
            .ok_or_else(|| Error::InternalError("Trick not complete".to_string()))?;

        // Determine winner using game rules
        let winner = trick.taker();

        // Add to current hand
        let trick_index = self
            .current_hand
            .tricks()
            .iter()
            .position(|t| t.is_none())
            .ok_or_else(|| Error::InternalError("No space for trick".to_string()))?;
        self.current_hand.add(trick, trick_index);

        // Start new trick
        self.current_hand
            .set_current_trick(OngoingTrick::new(winner));

        // Check if hand is complete (all cards played)
        if self.players.iter().all(|p| p.hand().is_empty()) {
            self.complete_hand(winner)
        } else {
            Ok(MoveEffect::TrickCompleted { winner })
        }
    }

    fn complete_hand(&mut self, last_trick_winner: PlayerId) -> Result<MoveEffect, Error> {
        // Finish the current hand.
        let hand = self
            .current_hand
            .finish()
            .ok_or_else(|| Error::InternalError("Hand not complete".to_string()))?;

        let (team1, team2) = TressetteRules::score_hand(&hand);
        self.score.0 += team1;
        self.score.1 += team2;

        // Store completed hand
        self.completed_hands.push(hand);

        // Store cards back into the deck
        self.collect_cards();

        // Check if game is over
        if TressetteRules::is_game_over(self.score) {
            Ok(MoveEffect::GameOver {
                trick_winner: last_trick_winner,
                final_score: self.score,
            })
        } else {
            self.deck.shuffle(&mut self.rng);
            self.initialize_new_hand();
            self.deal();
            Ok(MoveEffect::HandComplete {
                trick_winner: last_trick_winner,
                score: self.score,
            })
        }
    }

    /// Collects all cards from the last completed hand back into the deck.
    fn collect_cards(&mut self) {
        if let Some(last_hand) = self.completed_hands.last() {
            last_hand
                .tricks()
                .iter()
                .flat_map(|t| t.cards().iter())
                .for_each(|&c| self.deck.push(c));
        }
    }

    /// Deals cards to all players.
    fn deal(&mut self) {
        let mut to_deal_to = self.dealing_player + 1;

        while !self.deck.is_empty() {
            if let Some(cards) = self.deck.draw_n(5) {
                for card in cards {
                    self.players[to_deal_to.as_usize()].give(card);
                }
                to_deal_to.inc();
            }
        }
    }

    /// Resets the hand state and prepares for a new hand.
    fn initialize_new_hand(&mut self) {
        self.dealing_player.inc();
        self.current_hand = OngoingHand::new(self.dealing_player + 1);
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Suit;
    use crate::core::italian::ItalianRank;
    use proptest::prelude::*;

    #[test]
    fn new_game_starts_dealt() {
        let game = Game::new();
        assert_eq!(game.score(), (0, 0));
        assert_eq!(game.hands_completed(), 0);

        // Each player should have 10 cards
        for i in 0..PLAYERS {
            assert_eq!(
                game.hand(PlayerId::try_from(i).unwrap()).len(),
                10,
                "Player {} should have 10 cards",
                i
            );
        }
    }

    #[test]
    fn can_make_legal_move() {
        let mut game = Game::new();
        let legal = game.legal_cards();
        assert!(!legal.is_empty());

        let initial_player = game.current_player();
        let effect = game.play_card(legal[0]).unwrap();
        assert!(matches!(effect, MoveEffect::CardPlayed));
        assert_ne!(game.current_player(), initial_player);
    }

    #[test]
    fn cannot_make_illegal_move() {
        let mut game = Game::new();

        // Set up a specific scenario
        let hearts_ace = TressetteCard::new(ItalianRank::Ace, Suit::Hearts);
        let spades_two = TressetteCard::new(ItalianRank::Two, Suit::Spades);

        game.players[1].give(hearts_ace);
        game.players[1].give(spades_two);
        let mut ongoing_trick = OngoingTrick::new(PlayerId::PLAYER_0);
        ongoing_trick.play(hearts_ace);
        game.current_hand.set_current_trick(ongoing_trick);

        // Player has hearts, must play hearts
        let result = game.play_card(spades_two);
        assert!(matches!(result, Err(Error::MustFollowSuit { .. })));
    }

    #[test]
    fn game_records_history() {
        let mut game = Game::new();

        let legal1 = game.legal_cards();
        game.play_card(legal1[0]).unwrap();

        let legal2 = game.legal_cards();
        game.play_card(legal2[0]).unwrap();

        assert_eq!(game.history().len(), 2);
    }

    #[test]
    fn deck_is_reshuffled_between_hands() {
        let mut game = Game::from_seed(42);

        while game.hands_completed() == 0 {
            let legal = game.legal_cards();
            game.play_card(legal[0]).unwrap();
        }

        // Reconstruct exactly what `deal()` would have produced for hand 2
        // if the deck had NOT been reshuffled: `collect_cards` always pushes
        // the just-completed hand's cards back in a fixed (trick, player)
        // order, and `deal()` draws 5 at a time by popping from the end.
        let last_hand = game.completed_hands.last().unwrap();
        let mut unshuffled: Vec<TressetteCard> = last_hand
            .tricks()
            .iter()
            .flat_map(|t| t.cards().iter())
            .copied()
            .collect();

        let mut predicted: [Vec<TressetteCard>; PLAYERS] =
            [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
        let mut to_deal_to = game.dealing_player + 1;
        while !unshuffled.is_empty() {
            for _ in 0..5 {
                predicted[to_deal_to.as_usize()].push(unshuffled.pop().unwrap());
            }
            to_deal_to.inc();
        }

        // With a real shuffle, matching this zero-entropy prediction for even
        // one player is astronomically unlikely, let alone all four.
        for player in PlayerId::ALL {
            let actual = game.hand(player).to_vec();
            assert_ne!(actual, predicted[player.as_usize()], "player {player}");
        }
    }

    proptest! {
        #[test]
        fn legal_cards_lead_to_valid_game(num_cards in 0..=TressetteRules::HAND_SIZE * PLAYERS) {
            let mut game = Game::new();
            let mut cards_played = 0;
            let mut prev_hand_sizes = [TressetteRules::HAND_SIZE; PLAYERS];
            let mut last_hands_completed = 0;

            while cards_played < num_cards && !matches!(game.status(), Status::Finished { .. }) {
                let legal = game.legal_cards();
                if !legal.is_empty() {
                    game.play_card(legal[0]).unwrap();
                    cards_played += 1;

                    // Reset prev_hand_sizes if a new hand was dealt
                    if game.hands_completed() > last_hands_completed {
                        prev_hand_sizes = [TressetteRules::HAND_SIZE; PLAYERS];
                        last_hands_completed = game.hands_completed();
                    }

                    // Making sure the score doesn't go crazy
                    prop_assert!(game.score().0 <= 50);
                    prop_assert!(game.score().1 <= 50);

                    let mut total_hand_size = 0;
                    for (player, prev_size) in prev_hand_sizes.iter_mut().enumerate().take(PLAYERS) {
                        let hand_len = game.hand(PlayerId::try_from(player).unwrap()).len();
                        // Hand size should decrease every time.
                        prop_assert!(hand_len <= *prev_size);
                        *prev_size = hand_len;
                        total_hand_size += hand_len;
                    }
                    // Making sure new cards don't come out of oblivion
                    prop_assert!(total_hand_size + cards_played <= 40 * (game.hands_completed() + 1));
                } else {
                    break;
                }
            }
        }
    }
}
