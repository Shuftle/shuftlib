#![allow(missing_docs)]
#![allow(clippy::expect_used)]

use shuftlib::tressette::{Game, MoveEffect, Status};

#[test]
fn tressette_works() {
    let mut game = Game::new();

    while !matches!(game.status(), Status::Finished { .. }) {
        let legal_cards = game.legal_cards();
        assert!(
            !legal_cards.is_empty(),
            "Should always have legal cards when game is ongoing"
        );

        // Always pick the first legal card (simple strategy for testing)
        let chosen_card = legal_cards[0];
        let effect = game
            .play_card(chosen_card)
            .expect("Legal card should succeed");

        match effect {
            MoveEffect::CardPlayed => {
                // Continue playing
            }
            MoveEffect::TrickCompleted { winner: _ } => {
                // Trick completed, continue to next trick
            }
            MoveEffect::HandComplete {
                trick_winner: _,
                score,
            } => {
                // Hand completed, scores updated, new hand auto-dealt
                assert_eq!(
                    (score.0 + score.1) % 11,
                    0,
                    "Scores should always sum to multiple of 11"
                );
            }
            MoveEffect::GameOver {
                trick_winner: _,
                final_score,
            } => {
                // Game is over
                assert_eq!((final_score.0 + final_score.1) % 11, 0);
                assert_ne!(final_score.0, final_score.1);
                assert!(
                    final_score.0 >= shuftlib::tressette::SCORE_TO_WIN
                        || final_score.1 >= shuftlib::tressette::SCORE_TO_WIN
                );
            }
        }
    }

    // Verify final state
    if let Status::Finished { winner } = game.status() {
        assert!(winner.is_some(), "Tressette should never end in a draw");
    }
}
