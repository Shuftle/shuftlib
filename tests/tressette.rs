#![feature(generic_const_exprs)]
use shuftlib::{
    common::{
        cards::Deck,
        hands::{OngoingHand, OngoingTrick, Player, PlayerId, TrickTakingGame},
    },
    tressette::{self, TressetteCard, TressetteRules},
};

#[test]
#[allow(clippy::unwrap_used)]
fn tressette_works() {
    let mut first = true;
    let mut leading_suit = None;
    let first_to_play = PlayerId::new(0).unwrap();
    let mut score = (0, 0);
    let mut players = [
        Player::new(PlayerId::new(0).unwrap()),
        Player::new(PlayerId::new(1).unwrap()),
        Player::new(PlayerId::new(2).unwrap()),
        Player::new(PlayerId::new(3).unwrap()),
    ];
    let mut hands = Vec::new();

    while !TressetteRules::is_completed(score) {
        let mut ongoing_hand = OngoingHand::<TressetteRules>::new();
        let mut deck = Deck::italian();
        deck.shuffle();

        for (i, &card) in deck.iter().enumerate() {
            let player_index = (i / 5) % TressetteRules::PLAYERS;
            players[player_index].give(TressetteCard::from(card));
        }

        for trick_id in 0..TressetteRules::TRICKS {
            let mut ongoing_trick = OngoingTrick::<TressetteRules>::new(first_to_play);
            for _ in 0..TressetteRules::PLAYERS {
                let next_to_play = ongoing_trick.next_to_play();
                let playable = TressetteRules::playable(&players[*next_to_play], leading_suit);
                TressetteRules::play(&mut players[*next_to_play], playable[0], &mut ongoing_trick);

                if first {
                    leading_suit = Some(playable[0].suit());
                    first = !first;
                }
            }
            first = !first;
            ongoing_hand.add(ongoing_trick.finish().unwrap(), trick_id);
        }
        let hand = ongoing_hand.finish().unwrap();
        TressetteRules::compute_score(&hand, &mut score);
        hands.push(hand);
    }

    assert_eq!((score.0 + score.1) % 11, 0);
    assert_ne!(score.0, score.1);
    assert!(score.0 >= tressette::SCORE_TO_WIN || score.1 >= tressette::SCORE_TO_WIN);
}
