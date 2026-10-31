#![allow(missing_docs)]
#![allow(clippy::expect_used)]

use shuftlib::{
    cards::italian::{ItalianRank, Suit},
    engine::{Session, Viewer},
    tressette::{Config, Deal, Event, PerSeat, Seat, Tressette, TressetteCard},
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
