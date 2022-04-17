use crate::bitboards::{BitBoard, BLANK};
use crate::castling::CastlingRights;
use crate::chess_board_builder::BoardBuilder;
use crate::colors::{Color, COLORS_NUMBER};
use crate::errors::Error;
use crate::move_masks::{
    BETWEEN_TABLE, BISHOP_TABLE, KING_TABLE, KNIGHT_TABLE, PAWN_TABLE, QUEEN_TABLE, ROOK_TABLE,
};
use crate::pieces::{Piece, PieceType, NUMBER_PIECE_TYPES};
use crate::square::{Square, SQUARES_NUMBER};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
pub struct ChessBoard {
    pieces_mask: [BitBoard; NUMBER_PIECE_TYPES],
    colors_mask: [BitBoard; COLORS_NUMBER],
    combined_mask: BitBoard,
    side_to_move: Color,
    castle_rights: [CastlingRights; COLORS_NUMBER],
    en_passant: Option<Square>,
    pinned: BitBoard,
    checks: BitBoard,
    moves_since_capture_counter: usize,
    black_moved_counter: usize,
    hash: usize,
}

impl Hash for ChessBoard {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl TryFrom<&BoardBuilder> for ChessBoard {
    type Error = Error;

    fn try_from(builder: &BoardBuilder) -> Result<Self, Self::Error> {
        let mut board = ChessBoard::new();

        for i in 0..SQUARES_NUMBER {
            let square = Square::new(i as u8).unwrap();
            if let Some(piece) = builder[square] {
                unsafe { board.put_piece_unchecked(piece, square); }
            }
        }

        board.side_to_move = builder.get_side_to_move();

        if let Some(ep) = builder.get_en_passant() {
            board.side_to_move = !board.side_to_move;
            board.set_en_passant(Some(ep));
            board.side_to_move = !board.side_to_move;
        }

        board.set_castling_rights(Color::White, builder.get_castle_rights(Color::White));
        board.set_castling_rights(Color::Black, builder.get_castle_rights(Color::Black));
        board.set_black_moves_counter(builder.get_black_moved_counter());
        board.set_moves_since_capture_counter(builder.get_moves_since_capture());
        board.update_pins_and_checks();

        match board.validate() {
            None => Ok(board),
            Some(err) => Err(err),
        }
    }
}

impl TryFrom<&mut BoardBuilder> for ChessBoard {
    type Error = Error;

    fn try_from(fen: &mut BoardBuilder) -> Result<Self, Self::Error> {
        (&*fen).try_into()
    }
}

impl TryFrom<BoardBuilder> for ChessBoard {
    type Error = Error;

    fn try_from(fen: BoardBuilder) -> Result<Self, Self::Error> {
        (&fen).try_into()
    }
}

impl FromStr for ChessBoard {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(BoardBuilder::from_str(value)?.try_into()?)
    }
}

impl fmt::Display for ChessBoard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let builder = BoardBuilder::try_from(self).unwrap();
        write!(f, "{}", builder)
    }
}

impl Default for ChessBoard {
    #[inline]
    fn default() -> ChessBoard {
        ChessBoard::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }
}

impl ChessBoard {
    pub fn new() -> Self {
        ChessBoard {
            pieces_mask: [BLANK; NUMBER_PIECE_TYPES],
            colors_mask: [BLANK; COLORS_NUMBER],
            combined_mask: BLANK,
            side_to_move: Color::White,
            castle_rights: [CastlingRights::BothSides; COLORS_NUMBER],
            en_passant: None,
            pinned: BLANK,
            checks: BLANK,
            moves_since_capture_counter: 0,
            black_moved_counter: 0,
            hash: 0,
        }
    }

    pub fn validate(&self) -> Option<Error> {
        None
    }

    pub fn hash(&self) -> usize {
        todo!()
    }

    #[inline]
    pub fn get_color_mask(&self, color: Color) -> BitBoard {
        self.colors_mask[color.to_index()]
    }

    #[inline]
    pub fn get_combined_mask(&self) -> BitBoard {
        self.combined_mask
    }

    #[inline]
    pub fn get_piece_type_position(&self, piece_type: PieceType) -> BitBoard {
        unsafe { *self.pieces_mask.get_unchecked(piece_type.to_index()) }
    }

    #[inline]
    pub fn get_king_square(&self, color: Color) -> Square {
        (self.get_piece_type_position(PieceType::King) & self.colors_mask[color.to_index()])
            .to_square()
    }

    #[inline]
    pub fn get_piece_type_masks(&self, piece_type: PieceType) -> BitBoard {
        self.pieces_mask[piece_type.to_index()]
    }

    #[inline]
    pub fn get_black_moved_counter(&self) -> usize {
        self.black_moved_counter
    }

    #[inline]
    pub fn get_moves_since_capture(&self) -> usize {
        self.moves_since_capture_counter
    }

    #[inline]
    pub fn get_castle_rights(&self, color: Color) -> CastlingRights {
        self.castle_rights[color.to_index()]
    }

    #[inline]
    pub fn get_side_to_move(&self) -> Color {
        self.side_to_move
    }

    #[inline]
    pub fn get_en_passant(&self) -> Option<Square> {
        self.en_passant
    }

    #[inline]
    pub fn get_check_mask(&self) -> BitBoard {
        self.checks
    }

    pub fn get_piece_type_on(&self, square: Square) -> Option<PieceType> {
        let bitboard = BitBoard::from_square(square);
        if self.get_combined_mask() & bitboard == BLANK {
            None
        } else {
            if (self.get_piece_type_position(PieceType::Pawn)
                ^ self.get_piece_type_position(PieceType::Knight)
                ^ self.get_piece_type_position(PieceType::Bishop))
                & bitboard
                != BLANK
            {
                if self.get_piece_type_position(PieceType::Pawn) & bitboard != BLANK {
                    Some(PieceType::Pawn)
                } else if self.get_piece_type_position(PieceType::Knight) & bitboard != BLANK {
                    Some(PieceType::Knight)
                } else {
                    Some(PieceType::Bishop)
                }
            } else {
                if self.get_piece_type_position(PieceType::Rook) & bitboard != BLANK {
                    Some(PieceType::Rook)
                } else if self.get_piece_type_position(PieceType::Queen) & bitboard != BLANK {
                    Some(PieceType::Queen)
                } else {
                    Some(PieceType::King)
                }
            }
        }
    }

    pub fn get_piece_color_on(&self, square: Square) -> Option<Color> {
        if (self.get_color_mask(Color::White) & BitBoard::from_square(square)) != BLANK {
            Some(Color::White)
        } else if (self.get_color_mask(Color::Black) & BitBoard::from_square(square)) != BLANK {
            Some(Color::Black)
        } else {
            None
        }
    }

    pub fn set_black_moves_counter(&mut self, counter: usize) -> &mut Self {
        self.black_moved_counter = counter;
        self
    }

    pub fn set_moves_since_capture_counter(&mut self, counter: usize) -> &mut Self {
        self.moves_since_capture_counter = counter;
        self
    }

    pub fn set_side_to_move(&mut self, color: Color) -> &mut Self {
        self.side_to_move = color;
        self
    }

    pub fn set_castling_rights(&mut self, color: Color, rights: CastlingRights) -> &mut Self {
        self.castle_rights[color.to_index()] = rights;
        self
    }

    pub fn set_en_passant(&mut self, square: Option<Square>) -> *mut Self {
        self.en_passant = square;
        self
    }

    unsafe fn put_piece_unchecked(&mut self, piece: Piece, square: Square) {
        self.clear_square_unchecked(square);
        let square_bitboard = BitBoard::from_square(square);
        self.combined_mask |= square_bitboard;
        self.pieces_mask[piece.0.to_index()] |= square_bitboard;
        self.colors_mask[piece.1.to_index()] |= square_bitboard;
    }

    fn put_piece(&mut self, piece: Piece, square: Square) {
        unsafe { self.put_piece_unchecked(piece, square); }
        self.update_pins_and_checks();
        self.validate();
    }

    unsafe fn clear_square_unchecked(&mut self, square: Square) {
        match self.get_piece_type_on(square) {
            Some(piece_type) => {
                let color = self.get_piece_color_on(square).unwrap();
                let mask = !BitBoard::from_square(square);

                self.combined_mask &= mask;
                self.pieces_mask[piece_type.to_index()] &= mask;
                self.colors_mask[color.to_index()] &= mask;
            }
            None => {}
        }
    }

    fn clear_square(&mut self, square: Square) {
        unsafe { self.clear_square_unchecked(square); }
        self.update_pins_and_checks();  // TODO compare hashes for not to check if nothing changes
        self.validate();
    }

    #[rustfmt::skip]
    fn update_pins_and_checks(&mut self) {
        self.pinned = BLANK;
        self.checks = BLANK;

        let king_square = self.get_king_square(self.side_to_move);
        let pinners = self.get_color_mask(!self.side_to_move)
            & (
                BISHOP_TABLE.lock().unwrap().get_moves(king_square)
                    & (self.get_piece_type_masks(PieceType::Bishop) | self.get_piece_type_masks(PieceType::Queen))
                | ROOK_TABLE.lock().unwrap().get_moves(king_square)
                    & (self.get_piece_type_masks(PieceType::Rook) | self.get_piece_type_masks(PieceType::Queen))
            );

        for pinner_square in pinners {
            let between = self.get_combined_mask()
                & BETWEEN_TABLE
                    .lock()
                    .unwrap()
                    .get(king_square, pinner_square)
                    .unwrap();
            if between == BLANK {
                self.checks ^= BitBoard::from_square(pinner_square);
            } else if between.count_ones() == 1 {
                self.pinned ^= between;
            }
        }

        self.checks ^= KNIGHT_TABLE.lock().unwrap().get_moves(king_square)
            & self.get_piece_type_masks(PieceType::Knight);

        self.checks ^= PAWN_TABLE
            .lock()
            .unwrap()
            .get_captures(king_square, !self.side_to_move)
            & self.get_piece_type_masks(PieceType::Pawn);
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum BoardStatus {
    Ongoing,
    Stalemate,
    Checkmate,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_from_string() {
        assert_eq!(
            format!("{}", ChessBoard::default()),
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        );
    }
}
