use crate::bitboards::BitBoard;
use std::collections::HashMap;

pub type MoveTable = HashMap<BitBoard, BitBoard>;

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
