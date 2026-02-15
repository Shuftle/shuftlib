# Shuftlib

Shuftlib is a Rust library for implementing card games. It provides reusable components for deck management, shuffling, and game mechanics.

## Usage

```rust
use shuftlib::tressette::Game;

let mut game = Game::new();

while !matches!(game.status(), shuftlib::tressette::Status::Finished { .. }) {
    let legal_cards = game.legal_cards();
    let chosen_card = legal_cards[0];
    game.play_card(chosen_card).expect("Legal move");
}

if let shuftlib::tressette::Status::Finished { winner } = game.status() {
    println!("Winner: {:?}", winner);
}
```

## Documentation

Documentation is at [docs.rs/shuftlib](https://docs.rs/shuftlib).

## License

Licensed under either the MIT License or the Apache License 2.0.
