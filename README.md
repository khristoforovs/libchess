# libchess

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/khristoforovs/libchess/blob/master/LICENSE)

**libchess** is a high-performance chess library for Rust. It provides comprehensive chess functionality including board representation, move validation, chess notation support (FEN, PGN, SAN), and UCI engine integration.


## Key Features

- 🏁 **Efficient Board Representation**: 64-bit bitboard implementation
- ♟️ **Complete Move Generation**: Full chess rules support:
  - Special moves (castling, en passant, promotion)
  - Check / checkmate / stalemate detection
  - Attack and pin calculation
- 📜 **Notation Support**:
  - FEN (Forsyth-Edwards Notation)
  - PGN (Portable Game Notation)
  - SAN (Standard Algebraic Notation)
- ⚙️ **Advanced Functionality**:
  - Position repetition detection
  - Material evaluation
  - Move filtering capabilities


## Examples

### Initializing a ChessBoard:
The easiest way to initialize the board is to use the FEN-string. Also, if you
need a default starting chess position you can use the `::default()` method:

```rust
use libchess::ChessBoard;
use std::str::FromStr;

fn main() {
    println!("{}", ChessBoard::default()); // draw the starting chess position

    let fen = "8/P5k1/2b3p1/5p2/5K2/7R/8/8 w - - 13 61";
    let board = ChessBoard::from_str(fen).unwrap(); // the board could be initialized from fen-string
    println!("{}", board); // this will draw the board representation in terminal
}
```

![Board Rendering](./img/screenshot_render_straight.png)

*Here uppercase letters represent white's pieces and lowercase - black's.*

Another way to create board is to convert it from manually created `BoardBuilder` structure:

```rust
use libchess::*;
use libchess::{squares::*, Color::*, PieceType::*};

fn main() {
    let board = ChessBoard::setup(
        &[
            (E1, Piece(King, White)),
            (E8, Piece(King, Black)),
            (E2, Piece(Pawn, White)),
        ], // iterable container of pairs Square + Piece
        White, // side to move
        CastlingRights::Neither, // white castling rights
        CastlingRights::Neither, // black castling rights
        None, // Optional en-passant square
        0, // Moves number since last capture or pawn move
        1, // Move number
    )
    .unwrap();

    println!("{}", board);
}
```

Of course, you need to be ensured that this set of settings will generate valid board, in other case board validation method will return `Err(LibChessError)` while you call `builder.try_into()` (as it does if you try to initialize the board by an invalid FEN-string)


### Basic usage

Here is an example of generating a list of available moves in the current position. After getting such array it is shown how to create and apply a piece move.

```rust
use libchess::{ChessBoard, BoardMove, squares::*, PieceType::*, PieceMove, mv};

fn main() {
    // Create starting position
    let mut board = ChessBoard::default();
    
    // Generate legal moves
    let moves = board.get_legal_moves();
    println!("Available moves: {}", moves.len());
    
    // Make e2-e4 move
    let e2e4 = mv!(Queen, A1, A8);

    board.make_move(&e2e4);
    println!("Position after e4:\n{}", board);
}
```


### Rendering options:

```rust
println!("{}", board.render_flipped()); // or you can render this board from black's perspective (flipped)
println!("{}", board.as_fen()); // will return a FEN-string "8/P5k1/2b3p1/5p2/5K2/7R/8/8 w - - 13 61"
```


### Initializing a Game object:

```rust
use libchess::*;
use libchess::{PieceType::*, squares::*};

fn main() {
    // initializing the game is almost the same as for boards
    let mut game = Game::from_fen("3k4/3P4/4K3/8/8/8/8/8 w - - 0 1").unwrap();
    let moves = vec![mv!(King, E6, D6)]; // defining vec of chess moves to be applied to the board
    moves.into_iter().for_each(|one| {
        game.make_move(&Action::MakeMove(one)).unwrap();
    });

    println!("{}", game.get_position());
    assert_eq!(game.get_game_status(), GameStatus::Stalemate);
}
```


### Making moves:

```rust
use libchess::*;
use libchess::{PieceType::*, squares::*, Color::*};

fn main() {
    let mut game = Game::default();
    let moves = vec![
        mv!(Pawn, E2, E4),
        mv!(Pawn, E7, E5),
        mv!(Queen, D1, H5),
        mv!(King, E8, E7),
        mv!(Queen, H5, E5),
    ];

    moves.into_iter().for_each(|one| {
        game.make_move(&Action::MakeMove(one)).unwrap();
    });

    println!("{}", game.get_position());
    assert_eq!(game.get_game_status(), GameStatus::CheckMated(Black));
}
```

Also you can define moves by str: 
```rust
use libchess::*;
use libchess::{PieceType::*, squares::*};
use std::str::FromStr;

fn main() {
    let mut game = Game::default();
    let moves = vec![
        "e2e4", "c7c5",
        "Ng1f3", "d7d6",
        "d2d4", "c5d4",
        "Nf3d4", "Ng8f6",
    ];

    moves.into_iter().for_each(|one| {
        game.make_move(&Action::MakeMove(mv_str!(one))).unwrap();
    });
    println!("{}", game.get_position());
}
```


### Game history representation:


```rust
println!("{}", game.get_action_history());
```


## License
Distributed under the MIT License. See [LICENSE](https://github.com/khristoforovs/libchess/blob/master/LICENSE) for details.


## Contributing
- Pull requests and bug reports are welcome! Before contributing:
- Discuss features in issues
- Add tests for new functionality
- Update relevant documentation
- Verify formatting: cargo fmt


*This library was inspired by other interesting chess libraries written in Rust:*
- [Chess](https://github.com/jordanbray/chess)
- [Shakmaty](https://crates.io/crates/shakmaty)
