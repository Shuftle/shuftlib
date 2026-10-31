#![allow(missing_docs)]
#![allow(clippy::expect_used)]

use rand::{SeedableRng, rngs::StdRng, seq::IndexedRandom};
use shuftlib::{
    cards::italian::{ItalianRank, Suit},
    engine::{Observation, Session, Status, Viewer},
    tressette::{
        Config, Deal, Event, PerSeat, SCORE_TO_WIN, Seat, Tressette, TressetteCard,
        declaration::{self, Visibility, Window},
        signal::{self, Signal},
    },
};

#[test]
fn a_deal_made_with_real_cards_can_be_entered() {
    let suit = |suit| {
        ItalianRank::ALL
            .into_iter()
            .map(|rank| TressetteCard::new(rank, suit))
            .collect::<Vec<_>>()
    };
    let mut session = Session::<Tressette>::new(Config::default());
    session
        .resolve_chance_with(Deal {
            dealer: Seat::East,
            hands: PerSeat {
                north: suit(Suit::Coins),
                east: suit(Suit::Cups),
                south: suit(Suit::Swords),
                west: suit(Suit::Clubs),
            },
        })
        .expect("the deal is complete");
    assert_eq!(session.observe(Seat::South).hand(), suit(Suit::Swords));
    assert_eq!(session.status().to_act(), [Seat::North]);
    assert!(session.spectate().hand().is_empty());

    // A seat that reconnects gets back the deal, and the cards it was dealt.
    let updates = session
        .updates_for(Viewer::Seat(Seat::West), 0)
        .expect("the log replays");
    let received: Vec<_> = updates[0].for_seat(Seat::West).collect();
    assert_eq!(
        received,
        [
            &Event::HandDealt { dealer: Seat::East },
            &Event::CardsReceived {
                cards: suit(Suit::Clubs)
            },
        ]
    );
}

#[test]
fn whole_games_can_be_played_and_replayed_in_any_variant() {
    let variants = [
        // The default: accusi on the first three tricks, played without signals.
        Config::default(),
        // Accusi only on the first trick, and others learn just their points.
        Config {
            declarations: declaration::Rules {
                window: Window::FirstTrick,
                visibility: Visibility::Anonymous,
            },
            ..Config::default()
        },
        // Every signal allowed.
        Config {
            signals: Signal::ALL
                .into_iter()
                .fold(signal::Rules::default(), signal::Rules::allow),
            ..Config::default()
        },
    ];
    for config in variants {
        play_and_replay(config);
    }
}

/// Plays a whole game of `config`, every seat choosing among its legal
/// actions at random, and checks it ends with a winner and replays from its
/// log.
fn play_and_replay(config: Config) {
    let mut rng = StdRng::seed_from_u64(7);
    let mut session = Session::<Tressette>::new(config);
    let outcome = loop {
        match session.status() {
            Status::Chance => {
                session.resolve_chance(&mut rng).expect("a deal is drawn");
            }
            Status::ToAct(seats) => {
                let seat = seats[0];
                let legal = session.observe(seat).legal_actions();
                let action = *legal.choose(&mut rng).expect("a seat always has a move");
                session
                    .act(seat, action)
                    .expect("legal actions are accepted");
            }
            Status::Over(outcome) => break outcome,
        }
    };
    assert!(outcome.score[outcome.winner] >= SCORE_TO_WIN);

    // The log alone replays the same game.
    let replay =
        Session::<Tressette>::replay(config, session.log().to_vec()).expect("the log replays");
    assert_eq!(replay, session);
}
