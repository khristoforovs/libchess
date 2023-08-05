//! This module implements the fast game position hashing methodology (Zobrist hashing)
//!
//! Allows to calculate and to fast update the "unique" hash value for each position
//! Number of hash collisions grows like the square root of the number of positions
//! under consideration

use crate::{
    CastlingRights, ChessBoard, Color, Piece, Square, CASTLING_RIGHTS_NUMBER, COLORS_NUMBER,
    FILES_NUMBER, PIECE_TYPES_NUMBER, SQUARES_NUMBER,
};
use lazy_static::lazy_static;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

const SEED: u64 = 1370359990842121; // The most meaningful constant in my code.
                                    // And in any other's code too, actually

pub type PositionHashValueType = u64;

#[derive(Debug, Clone)]
pub struct ZobristHasher {
    piece_square_table:
        [[[PositionHashValueType; SQUARES_NUMBER]; PIECE_TYPES_NUMBER]; COLORS_NUMBER],
    castling_table:      [[PositionHashValueType; CASTLING_RIGHTS_NUMBER]; COLORS_NUMBER],
    en_passant_table:    [PositionHashValueType; FILES_NUMBER],
    black_to_move_value: PositionHashValueType,
}

impl Default for ZobristHasher {
    fn default() -> Self { Self::new() }
}

impl ZobristHasher {
    pub fn new() -> Self {
        let mut result = Self {
            piece_square_table:  [[[0; SQUARES_NUMBER]; PIECE_TYPES_NUMBER]; COLORS_NUMBER],
            castling_table:      [[0; CASTLING_RIGHTS_NUMBER]; COLORS_NUMBER],
            en_passant_table:    [0; FILES_NUMBER],
            black_to_move_value: 0,
        };

        result.generate_tables();
        result
    }

    fn generate_tables(&mut self) -> &mut Self {
        let mut rng = StdRng::seed_from_u64(SEED);

        // side to move
        self.black_to_move_value = rng.gen();

        // fill table for pieces positions
        for c in 0..COLORS_NUMBER {
            for p in 0..PIECE_TYPES_NUMBER {
                for sq in 0..SQUARES_NUMBER {
                    self.piece_square_table[c][p][sq] = rng.gen();
                }
            }
        }

        // fill table for castling
        for c in 0..COLORS_NUMBER {
            for r in 0..CASTLING_RIGHTS_NUMBER {
                self.castling_table[c][r] = rng.gen();
            }
        }

        // fill table for en passant
        for f in 0..FILES_NUMBER {
            self.en_passant_table[f] = rng.gen();
        }

        self
    }

    pub fn calculate_position_hash(&self, position: &ChessBoard) -> PositionHashValueType {
        let mut hash = 0;

        // side to move
        if Color::Black == position.get_side_to_move() {
            hash ^= self.black_to_move_value;
        }

        // pieces positions
        for sq in position.get_combined_mask() {
            let piece_type = position.get_piece_type_on(sq).unwrap();
            let color = position.get_piece_color_on(sq).unwrap();
            hash ^= self.piece_square_table[color.to_index()][piece_type.to_index()][sq.to_index()];
        }

        // castling
        for color in [Color::White, Color::Black] {
            hash ^=
                self.castling_table[color.to_index()][position.get_castle_rights(color).to_index()];
        }

        // en passant
        if let Some(sq) = position.get_en_passant() {
            hash ^= self.en_passant_table[sq.get_file().to_index()];
        }

        hash
    }

    pub fn get_piece_square_value(&self, piece: Piece, square: Square) -> PositionHashValueType {
        self.piece_square_table[piece.1.to_index()][piece.0.to_index()][square.to_index()]
    }

    pub fn get_black_to_move_value(&self) -> PositionHashValueType { self.black_to_move_value }

    pub fn get_castling_rights_value(
        &self,
        castling_rights: CastlingRights,
        color: Color,
    ) -> PositionHashValueType {
        self.castling_table[color.to_index()][castling_rights.to_index()]
    }

    pub fn get_en_passant_value(&self, square: Square) -> PositionHashValueType {
        self.en_passant_table[square.get_file().to_index()]
    }
}

lazy_static! {
    pub static ref ZOBRIST_TABLES: ZobristHasher = ZobristHasher::new();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mv;
    use crate::PieceType::*;
    use crate::{squares::*, BoardMove, PieceMove, ZOBRIST_TABLES as ZOBRIST};

    #[test]
    fn calculate_hash() {
        let board = ChessBoard::default();

        let chess_move = mv!(Pawn, E2, E4);
        let new_board = board.make_move(&chess_move).unwrap();

        assert_ne!(
            ZOBRIST.calculate_position_hash(&board),
            ZOBRIST.calculate_position_hash(&new_board)
        );

        let direct_calculated_hash = ZOBRIST.calculate_position_hash(&new_board);
        let live_updating_hash = new_board.get_hash();
        assert_eq!(direct_calculated_hash, live_updating_hash);
    }
}
