# Shuftlib

Shuftlib is a Rust library for implementing turn-based games, with tools for
card games.

Every game is a state machine implementing the `Game` trait. A `Session`
drives it: it checks who may act, records every step so
a game can be replayed from its log alone, and only lets players see what
they're allowed to know.

- **Chance is explicit.** A chance step, such as a deal, has an outcome you
  can inspect: the host either has the game draw one from a random number
  generator it supplies, or gives a specific one, e.g. a deal made with real
  cards.
- **Hidden information stays hidden.** Each seat, and each spectator, gets an
  observation of what it may know, and every step returns a public notice
  plus events addressed to everyone or to a single seat. A seat that
  reconnects can get every update it missed.
- **Hosts stay in control.** The engine never reads the time: time limits,
  e.g. for online play, are up to the host, which can forfeit a seat that
  takes too long.

The library ships with `tressette`: Tressette for four players in two
partnerships.

## Usage

```rust
use std::error::Error;

use shuftlib::{
    engine::{Observation, Session, Status},
    tressette::{Config, Tressette},
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut rng = rand::rng();
    let mut session = Session::<Tressette>::new(Config::default());
    let outcome = loop {
        match session.status() {
            // Waiting for a chance outcome: a new hand must be dealt.
            Status::Chance => {
                session.resolve_chance(&mut rng)?;
            }
            // Waiting for seats to act. Tressette waits on one at a time: play
            // its first legal card.
            Status::ToAct(seats) => {
                let seat = seats[0];
                let view = session.observe(seat);
                let action = view.legal_actions().into_iter().next().ok_or("no legal card")?;
                session.act(seat, action)?;
            }
            Status::Over(outcome) => break outcome,
        }
    };
    println!("{:?} won {:?}", outcome.winner, outcome.score);

    // The log holds every deal and every card played: it replays on its own.
    let replay = Session::<Tressette>::replay(Config::default(), session.log().to_vec())?;
    assert_eq!(replay, session);
    Ok(())
}
```

## Documentation

Documentation is at [docs.rs/shuftlib](https://docs.rs/shuftlib).

## License

Licensed under the GNU Affero General Public License v3.0 (AGPL-3.0-only).

See the [LICENSE](https://github.com/Shuftle/shuftlib/blob/main/LICENSE) file for details.
