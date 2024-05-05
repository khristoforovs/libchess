# libchess: Rust chess library

This library implements the game of chess: chess board, pieces, rules and legal moves.

What libchess can be used for:

- [x] Parse / write FEN-string
- [x] Represent / render the chess board
- [x] View and set board properties like castling rights
- [x] Generate legal moves
- [x] Make moves
- [x] Recognize terminals on the board (stalemate, checkmate, insufficient material draws, 50-moves draws)
- [x] Parse / write PGN-files
- [x] Represent the chess game
- [x] Recognize game terminals on the board (all the same as for the chess board but adding repetition draws, draws by agreement, resignations)
- [x] Rendering game moves history


## Examples


### Initializing a ChessBoard:
The easiest way to initialize the board is to use the FEN-string. Also, if you
need a default starting chess position you can use the `::default()` method:
```rust
use libchess::ChessBoard; 

println!("{}", ChessBoard::default()); // draw the starting chess position

let fen = "8/P5k1/2b3p1/5p2/5K2/7R/8/8 w - - 13 61";
let board = ChessBoard::from_str(fen).unwrap(); // the board could be initialized from fen-string
println!("{}", board); // this will draw the board representation in terminal:
```

![Board Rendering](./img/screenshot_render_straight.png)

*Here uppercase letters represent white's pieces and lowercase - black's.*

Another way to create board is to convert it from manually created `BoardBuilder` structure:

```rust
use libchess::*;
use libchess::{squares::*, Color::*, PieceType::*};

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
```

Of course, you need to be ensured that this set of settings will generate valid board, in other case board validation method will return `Err(LibChessError)` while you call `builder.try_into()` (as it does if you try to initialize the board by an invalid FEN-string)

### Rendering options:

```rust
println!("{}", board.render_flipped()); // or you can render this board from black's perspective (flipped)
println!("{}", board.as_fen()); // will return a FEN-string "8/P5k1/2b3p1/5p2/5K2/7R/8/8 w - - 13 61"
```


### Initializing a Game object:
```rust
use libchess::*;
use libchess::{PieceType::*, squares::*};

// initializing the game is almost the same as for boards
let mut game = Game::from_fen("3k4/3P4/4K3/8/8/8/8/8 w - - 0 1").unwrap();
let moves = vec![mv!(King, E6, D6)]; // defining vec of chess moves to be applied to the board
moves.into_iter().for_each(|one| {
    game.make_move(&Action::MakeMove(one)).unwrap();
});
assert_eq!(game.get_game_status(), GameStatus::Stalemate);
```


### Making moves:
```rust
use libchess::*;
use libchess::{PieceType::*, squares::*, Color::*};

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
assert_eq!(game.get_game_status(), GameStatus::CheckMated(Black));
```

Also you can define moves by str: 
```rust
use libchess::*;
use libchess::{PieceType::*, squares::*};
use std::str::FromStr;

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
```


### Game history representation:
```rust
println!("{}", game.get_action_history());
```


*This library was inspired by other interesting chess libraries written in Rust:*
* [Chess](https://github.com/jordanbray/chess)
* [Shakmaty](https://crates.io/crates/shakmaty)
