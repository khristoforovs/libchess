# libchess: Rust chess library

 
This library implements the game of chess: chess board, pieces, rules and legal moves.

## Examples

### Initializing a ChessBoard:
The easiest way to initialize the board is to use the FEN-string. Also, if you
need a default starting chess position you can use the default method:
```rust
use libchess::boards::ChessBoard; 
println!("{}", ChessBoard::default());

let board = ChessBoard::from_str("8/P5k1/2b3p1/5p2/5K2/7R/8/8 w - - 13 61").unwrap();
println!("{}", board);
println!("{}", board.as_fen());
```

### Making moves:
```rust
use libchess::{Game, Action, GameStatus, Color};
use libchess::boards::{ChessBoard, BoardMove, BoardMoveOption, PieceMove, squares::*};
use libchess::{castle_king_side, castle_queen_side, mv};
use libchess::PieceType::*;

let mut game = Game::default();
let moves = vec![
   mv!(Pawn, E2, E4),
   mv!(Pawn, E7, E5),
   mv!(Queen, D1, H5),
   mv!(King, E8, E7),
   mv!(Queen, H5, E5),
];

for one in moves.iter() {
    game.make_move(Action::MakeMove(*one)).unwrap();
}
assert_eq!(game.get_game_status(), GameStatus::CheckMated(Color::Black));
```

### Game history representation:
```rust
println!("{}", game.get_action_history());
```


*This library was inspired by other interesting and, obviously, more powerful chess libraries written in Rust:*
* [Chess](https://github.com/jordanbray/chess)
* [Shakmaty](https://crates.io/crates/shakmaty)
