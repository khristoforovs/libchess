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
