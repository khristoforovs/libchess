use crate::bitboards::{BitBoard, BLANK};
use crate::square::{Square, SQUARES_NUMBER};

pub struct MoveTable([BitBoard; SQUARES_NUMBER]);

impl MoveTable {
    pub fn new() -> Self {
        Self([BLANK; SQUARES_NUMBER])
    }

    pub fn set(&mut self, square: Square, value: BitBoard) {
        self.0[square.to_index()] = value;
    }

    pub fn get(&self, square: Square) -> BitBoard {
        self.0[square.to_index()]
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
