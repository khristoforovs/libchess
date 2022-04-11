use crate::bitboards::{BitBoard, BLANK};
use crate::square::{Square, SQUARES_NUMBER};

pub struct MoveTable {
    moves: [BitBoard; SQUARES_NUMBER],
    captures: Option<[BitBoard; SQUARES_NUMBER]>,
}

impl MoveTable {
    pub fn new() -> Self {
        Self {
            moves: [BLANK; SQUARES_NUMBER],
            captures: None,
        }
    }

    pub fn set_moves(&mut self, square: Square, value: BitBoard) {
        self.moves[square.to_index()] = value;
    }

    pub fn set_captures(&mut self, square: Square, value: BitBoard) {
        let mut captures = match self.captures {
            Some(captures) => captures,
            None => [BLANK; SQUARES_NUMBER],
        };
        captures[square.to_index()] = value;
        self.captures = Some(captures);
    }

    pub fn reset_captures(&mut self) {
        self.captures = None
    }

    pub fn get_moves(&self, square: Square) -> BitBoard {
        self.moves[square.to_index()]
    }

    pub fn get_captures(&self, square: Square) -> BitBoard {
        match self.captures {
            Some(captures) => captures[square.to_index()],
            None => self.moves[square.to_index()],
        }
    }
}

mod bishops;
pub use bishops::generate_bishop_moves;

mod knights;
pub use knights::generate_knight_moves;

mod rooks;
pub use rooks::generate_rook_moves;

mod kings;
pub use kings::generate_king_moves;

mod queens;
pub use queens::generate_queen_moves;

mod pawns;
pub use pawns::{generate_pawn_captures, generate_pawn_moves};

mod between;
pub use between::generate_between_masks;
