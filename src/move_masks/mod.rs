use crate::{BitBoard, Color, Square, BLANK, SQUARES_NUMBER};
use lazy_static::lazy_static;

pub struct PieceMoveTable([BitBoard; SQUARES_NUMBER]);

impl PieceMoveTable {
    pub fn new() -> Self { Self([BLANK; SQUARES_NUMBER]) }

    pub fn set_moves(&mut self, square: Square, value: BitBoard) {
        self.0[square.to_index()] = value;
    }

    pub fn reset_moves(&mut self) { self.0 = [BLANK; SQUARES_NUMBER]; }

    pub fn get_moves(&self, square: Square) -> BitBoard { self.0[square.to_index()] }
}

impl Default for PieceMoveTable {
    fn default() -> Self { Self::new() }
}

mod rays;
pub use rays::RaysTable;

mod bishops;
use bishops::generate_bishop_moves;

mod knights;
use knights::generate_knight_moves;

mod rooks;
use rooks::generate_rook_moves;

mod kings;
use kings::generate_king_moves;

mod queens;
use queens::generate_queen_moves;

mod pawns;
use pawns::{generate_pawn_captures, generate_pawn_moves, PawnMoveTable};

mod between;
use between::{generate_between_masks, BetweenTable};

lazy_static! {
    pub static ref RAYS_TABLE: RaysTable = RaysTable::default();
    pub static ref BISHOP_TABLE: PieceMoveTable = {
        let mut table = PieceMoveTable::new();
        generate_bishop_moves(&mut table);
        table
    };
    pub static ref KNIGHT_TABLE: PieceMoveTable = {
        let mut table = PieceMoveTable::new();
        generate_knight_moves(&mut table);
        table
    };
    pub static ref ROOK_TABLE: PieceMoveTable = {
        let mut table = PieceMoveTable::new();
        generate_rook_moves(&mut table);
        table
    };
    pub static ref QUEEN_TABLE: PieceMoveTable = {
        let mut table = PieceMoveTable::new();
        generate_queen_moves(&mut table);
        table
    };
    pub static ref KING_TABLE: PieceMoveTable = {
        let mut table = PieceMoveTable::new();
        generate_king_moves(&mut table);
        table
    };
    pub static ref PAWN_TABLE: PawnMoveTable = {
        let mut table = PawnMoveTable::new();
        generate_pawn_moves(&mut table, Color::White);
        generate_pawn_moves(&mut table, Color::Black);
        generate_pawn_captures(&mut table, Color::White);
        generate_pawn_captures(&mut table, Color::Black);
        table
    };
    pub static ref BETWEEN_TABLE: BetweenTable = {
        let mut between_table = BetweenTable::new();
        generate_between_masks(&mut between_table);
        between_table
    };
}
