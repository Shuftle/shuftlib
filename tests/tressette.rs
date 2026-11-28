#![allow(missing_docs)]

use anyhow::{Context, ensure};
use shuftlib::{
    cards::italian::{ItalianRank, Suit},
    engine::{Session, Viewer},
    tressette::{Config, Deal, Event, PerSeat, Seat, Tressette, TressetteCard},
};

#[test]
fn a_deal_made_with_real_cards_can_be_entered() -> anyhow::Result<()> {
    let suit = |suit| {
        ItalianRank::ALL
            .into_iter()
            .map(|rank| TressetteCard::new(rank, suit))
            .collect::<Vec<_>>()
    };
    let deal = Deal::new(
        Seat::East,
        PerSeat {
            north: suit(Suit::Coins),
            east: suit(Suit::Cups),
            south: suit(Suit::Swords),
            west: suit(Suit::Clubs),
        },
    )?;
    let mut session = Session::<Tressette>::new(Config::default());
    session.resolve_chance_with(deal)?;
    ensure!(session.observe(Seat::South).hand() == suit(Suit::Swords));
    ensure!(session.status().to_act() == [Seat::North]);
    ensure!(session.spectate().hand().is_empty());

    // A seat that reconnects gets back the deal, and the cards it was dealt.
    let updates = session.updates_for(Viewer::Seat(Seat::West), 0)?;
    let received: Vec<_> = updates
        .first()
        .context("the log replays the deal")?
        .for_seat(Seat::West)
        .collect();
    ensure!(
        received
            == [
                &Event::HandDealt { dealer: Seat::East },
                &Event::CardsReceived {
                    cards: suit(Suit::Clubs)
                },
            ]
    );
    Ok(())
}
