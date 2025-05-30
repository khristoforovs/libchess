use libchess::ChessBoard;
use std::str::FromStr;

fn main() {
    println!("{}", ChessBoard::default()); // draw the starting chess position

    let fen = "8/P5k1/2b3p1/5p2/5K2/7R/8/8 w - - 13 61";
    let board = ChessBoard::from_str(fen).unwrap(); // the board could be initialized from fen-string
    println!("{}", board); // this will draw the board representation in terminal
}
