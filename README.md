# Shuftlib

Shuftlib is a Rust library for implementing turn-based games.

Every game is a state machine implementing the `Game` trait. A `Session`
drives it: it checks who may act, records every step so
a game can be replayed from its log alone and only lets players see what
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

## Games

Each game's module documents its rules and how to play it, with a full example.

- `tressette`: Tressette for four players in two partnerships, with accusi and
  signals set per regional variant.

## Documentation

Documentation is at [docs.rs/shuftlib](https://docs.rs/shuftlib).

## License

Licensed under the GNU Affero General Public License v3.0 (AGPL-3.0-only).

See the [LICENSE](https://github.com/Shuftle/shuftlib/blob/main/LICENSE) file for details.
