use crate::bitboards::{BitBoard, BLANK};
use crate::board_files::FILES;
use crate::board_ranks::{Rank, RANKS};
use crate::castling::CastlingRights;
use crate::chess_board_builder::BoardBuilder;
use crate::chess_moves::{ChessMove, PromotionPieceType};
use crate::colors::{Color, COLORS_NUMBER};
use crate::errors::ChessBoardError as Error;
use crate::move_masks::{
    BETWEEN_TABLE as BETWEEN, BISHOP_TABLE as BISHOP, KING_TABLE as KING, KNIGHT_TABLE as KNIGHT,
    PAWN_TABLE as PAWN, QUEEN_TABLE as QUEEN, ROOK_TABLE as ROOK,
};
use crate::mv;
use crate::pieces::{Piece, PieceType, NUMBER_PIECE_TYPES};
use crate::squares::{Square, SQUARES_NUMBER};
use colored::Colorize;
use either::Either;
use std::collections::hash_map::DefaultHasher;
use std::collections::hash_set::HashSet;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

pub type LegalMoves = HashSet<ChessMove>;

#[derive(Debug, Clone, PartialEq, Eq)]
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
    flipped_view: bool,
    legal_moves: LegalMoves,
}

impl Hash for ChessBoard {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pieces_mask.hash(state);
        self.colors_mask.hash(state);
        self.side_to_move.hash(state);
        self.castle_rights.hash(state);
        self.en_passant.hash(state);
        self.moves_since_capture_counter.hash(state);
        self.black_moved_counter.hash(state);
    }
}

impl TryFrom<&BoardBuilder> for ChessBoard {
    type Error = Error;

    fn try_from(builder: &BoardBuilder) -> Result<Self, Self::Error> {
        let mut board = ChessBoard::new();

        for i in 0..SQUARES_NUMBER {
            let square = Square::new(i as u8).unwrap();
            if let Some(piece) = builder[square] {
                board.put_piece_unchecked(piece, square);
            }
        }

        board.side_to_move = builder.get_side_to_move();
        board.set_en_passant(builder.get_en_passant());
        board.set_castling_rights(Color::White, builder.get_castle_rights(Color::White));
        board.set_castling_rights(Color::Black, builder.get_castle_rights(Color::Black));
        board.set_black_moves_counter(builder.get_black_moved_counter());
        board.set_moves_since_capture_counter(builder.get_moves_since_capture());
        board.update_pins_and_checks();
        board.update_legal_moves();

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
        let ranks = if self.flipped_view {
            Either::Left(RANKS.iter())
        } else {
            Either::Right(RANKS.iter().rev())
        };
        let files = if self.flipped_view {
            Either::Right(FILES.iter().rev())
        } else {
            Either::Left(FILES.iter())
        };
        let footer = if self.flipped_view {
            "     h  g  f  e  d  c  b  a"
        } else {
            "     a  b  c  d  e  f  g  h"
        };

        let mut field_string = String::new();
        for rank in ranks {
            field_string = format!("{}{}  ║", field_string, (*rank).to_index() + 1);
            for file in files.clone() {
                let square = Square::from_rank_file(*rank, *file);
                if self.is_empty_square(square) {
                    if square.is_light() {
                        field_string = format!("{}{}", field_string, "   ".on_white());
                    } else {
                        field_string = format!("{}{}", field_string, "   ");
                    };
                } else {
                    let mut piece_type_str =
                        format!(" {} ", self.get_piece_type_on(square).unwrap());
                    piece_type_str = match self.get_piece_color_on(square).unwrap() {
                        Color::White => piece_type_str.to_uppercase(),
                        Color::Black => piece_type_str.to_lowercase(),
                    };
                    if square.is_light() {
                        field_string =
                            format!("{}{}", field_string, piece_type_str.black().on_white());
                    } else {
                        field_string = format!("{}{}", field_string, piece_type_str);
                    };
                }
            }
            field_string = format!("{}║\n", field_string);
        }

        let board_string = format!(
            "   {}  {}{}\n{}\n{}{}\n{}\n",
            self.get_side_to_move(),
            format!("{}", self.get_castle_rights(Color::White)).to_uppercase(),
            self.get_castle_rights(Color::Black),
            "   ╔════════════════════════╗",
            field_string,
            "   ╚════════════════════════╝",
            footer,
        );
        write!(f, "{}", board_string)
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
        let mut result = ChessBoard {
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
            flipped_view: false,
            legal_moves: LegalMoves::new(),
        };

        result.update_legal_moves();
        result
    }

    pub fn validate(&self) -> Option<Error> {
        // make sure that is no color overlapping
        if self.get_color_mask(Color::White) & self.get_color_mask(Color::Black) != BLANK {
            return Some(Error::InvalidPositionColorsOverlap);
        };

        // check overlapping of piece type masks
        for i in 0..(NUMBER_PIECE_TYPES - 1) {
            for j in i + 1..NUMBER_PIECE_TYPES {
                if (self.get_piece_type_mask(PieceType::from_index(i).unwrap())
                    & self.get_piece_type_mask(PieceType::from_index(j).unwrap()))
                    != BLANK
                {
                    return Some(Error::InvalidPositionPieceTypeOverlap);
                }
            }
        }

        // make sure that each square has only 0 or 1 piece
        let calculated_combined = {
            (0..NUMBER_PIECE_TYPES).fold(BLANK, |current, i| {
                current | self.get_piece_type_mask(PieceType::from_index(i).unwrap())
            })
        };
        if calculated_combined != self.get_combined_mask() {
            return Some(Error::InvalidBoardSelfNonConsistency);
        }

        // make sure there is 1 black and 1 white king
        let king_mask = self.get_piece_type_mask(PieceType::King);
        if (king_mask & self.get_color_mask(Color::White)).count_ones() != 1 {
            return Some(Error::InvalidBoardMultipleOneColorKings);
        }
        if (king_mask & self.get_color_mask(Color::White)).count_ones() != 1 {
            return Some(Error::InvalidBoardMultipleOneColorKings);
        }

        // make sure that opponent is not on check
        let mut cloned_board = self.clone();
        cloned_board.set_side_to_move(!self.side_to_move);
        cloned_board.update_pins_and_checks();
        if cloned_board.get_check_mask().count_ones() > 0 {
            return Some(Error::InvalidBoardOpponentIsOnCheck);
        }

        // validate en passant
        match self.get_en_passant() {
            None => {}
            Some(square) => {
                if self.get_piece_type_mask(PieceType::Pawn)
                    & self.get_color_mask(!self.side_to_move)
                    & BitBoard::from_square(match !self.side_to_move {
                        Color::White => square.up().unwrap(),
                        Color::Black => square.down().unwrap(),
                    })
                    == BLANK
                {
                    return Some(Error::InvalidBoardInconsistentEnPassant);
                }
            }
        }

        // validate castling rights
        let rook_mask =
            self.get_piece_type_mask(PieceType::Rook) & self.get_color_mask(Color::White);
        let king_square = self.get_king_square(Color::White);
        match self.get_castle_rights(Color::White) {
            CastlingRights::Neither => {}
            CastlingRights::QueenSide => {
                if (king_square != Square::E1)
                    & (rook_mask & BitBoard::from_square(Square::A1) != BLANK)
                {
                    return Some(Error::InvalidBoardInconsistentCastlingRights);
                }
            }
            CastlingRights::KingSide => {
                if (king_square != Square::E1)
                    & (rook_mask & BitBoard::from_square(Square::H1) != BLANK)
                {
                    return Some(Error::InvalidBoardInconsistentCastlingRights);
                }
            }
            CastlingRights::BothSides => {
                if (king_square != Square::E1)
                    & (rook_mask & BitBoard::from_square(Square::A1) != BLANK)
                    & (rook_mask & BitBoard::from_square(Square::H1) != BLANK)
                {
                    return Some(Error::InvalidBoardInconsistentCastlingRights);
                }
            }
        }

        let rook_mask =
            self.get_piece_type_mask(PieceType::Rook) & self.get_color_mask(Color::Black);
        let king_square = self.get_king_square(Color::Black);
        match self.get_castle_rights(Color::Black) {
            CastlingRights::Neither => {}
            CastlingRights::QueenSide => {
                if (king_square != Square::E8)
                    & (rook_mask & BitBoard::from_square(Square::A8) != BLANK)
                {
                    return Some(Error::InvalidBoardInconsistentCastlingRights);
                }
            }
            CastlingRights::KingSide => {
                if (king_square != Square::E8)
                    & (rook_mask & BitBoard::from_square(Square::H8) != BLANK)
                {
                    return Some(Error::InvalidBoardInconsistentCastlingRights);
                }
            }
            CastlingRights::BothSides => {
                if (king_square != Square::E8)
                    & (rook_mask & BitBoard::from_square(Square::A8) != BLANK)
                    & (rook_mask & BitBoard::from_square(Square::H8) != BLANK)
                {
                    return Some(Error::InvalidBoardInconsistentCastlingRights);
                }
            }
        }

        None
    }

    #[inline]
    pub fn as_fen(&self) -> String {
        format!("{}", BoardBuilder::try_from(self).unwrap())
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
    pub fn get_king_square(&self, color: Color) -> Square {
        (self.get_piece_type_mask(PieceType::King) & self.get_color_mask(color)).to_square()
    }

    #[inline]
    pub fn get_piece_type_mask(&self, piece_type: PieceType) -> BitBoard {
        self.pieces_mask[piece_type.to_index()]
    }

    #[inline]
    pub fn get_pin_mask(&self) -> BitBoard {
        self.pinned
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

    #[inline]
    pub fn is_empty_square(&self, square: Square) -> bool {
        let mask = self.combined_mask & BitBoard::from_square(square);
        if mask.count_ones() == 0 {
            return true;
        };
        false
    }

    #[inline]
    pub fn get_flipped_view(&mut self) -> bool {
        self.flipped_view
    }

    pub fn set_flipped_view(&mut self, flipped: bool) {
        self.flipped_view = flipped
    }

    pub fn get_piece_type_on(&self, square: Square) -> Option<PieceType> {
        let bitboard = BitBoard::from_square(square);
        if self.get_combined_mask() & bitboard == BLANK {
            None
        } else {
            if (self.get_piece_type_mask(PieceType::Pawn)
                | self.get_piece_type_mask(PieceType::Knight)
                | self.get_piece_type_mask(PieceType::Bishop))
                & bitboard
                != BLANK
            {
                if self.get_piece_type_mask(PieceType::Pawn) & bitboard != BLANK {
                    Some(PieceType::Pawn)
                } else if self.get_piece_type_mask(PieceType::Knight) & bitboard != BLANK {
                    Some(PieceType::Knight)
                } else {
                    Some(PieceType::Bishop)
                }
            } else {
                if self.get_piece_type_mask(PieceType::Rook) & bitboard != BLANK {
                    Some(PieceType::Rook)
                } else if self.get_piece_type_mask(PieceType::Queen) & bitboard != BLANK {
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

    pub fn get_legal_moves(&self) -> &LegalMoves {
        &self.legal_moves
    }

    pub fn get_hash(&self) -> u64 {
        let mut h = DefaultHasher::new();
        self.hash(&mut h);
        h.finish()
    }

    pub fn make_move(&self, next_move: ChessMove) -> Result<Self, Error> {
        let mut next_position = self.clone();
        if self.get_legal_moves().contains(&next_move) {
            // TODO castling processing
            // TODO Promotion processing

            next_position
                .apply_move_unchecked(next_move)
                .set_side_to_move(!self.side_to_move)
                .update_pins_and_checks()
                .update_legal_moves();
        } else {
            return Err(Error::IllegalMoveDetected);
        }
        Ok(next_position)
    }

    fn apply_move_unchecked(&mut self, next_move: ChessMove) -> &mut Self {
        let side_to_move = self.get_side_to_move();
        self.clear_square_unchecked(next_move.get_source_square())
            .clear_square_unchecked(next_move.get_destination_square())
            .put_piece_unchecked(
                Piece(next_move.get_piece_type(), side_to_move),
                next_move.get_destination_square(),
            )
    }

    fn set_black_moves_counter(&mut self, counter: usize) -> &mut Self {
        self.black_moved_counter = counter;
        self
    }

    fn set_moves_since_capture_counter(&mut self, counter: usize) -> &mut Self {
        self.moves_since_capture_counter = counter;
        self
    }

    fn set_side_to_move(&mut self, color: Color) -> &mut Self {
        self.side_to_move = color;
        self
    }

    fn set_castling_rights(&mut self, color: Color, rights: CastlingRights) -> &mut Self {
        self.castle_rights[color.to_index()] = rights;
        self
    }

    fn set_en_passant(&mut self, square: Option<Square>) -> &mut Self {
        self.en_passant = square;
        self
    }

    fn put_piece_unchecked(&mut self, piece: Piece, square: Square) -> &mut Self {
        self.clear_square_unchecked(square);
        let square_bitboard = BitBoard::from_square(square);
        self.combined_mask |= square_bitboard;
        self.pieces_mask[piece.0.to_index()] |= square_bitboard;
        self.colors_mask[piece.1.to_index()] |= square_bitboard;
        self
    }

    fn clear_square_unchecked(&mut self, square: Square) -> &mut Self {
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
        self
    }

    fn update_pins_and_checks(&mut self) -> &mut Self {
        self.pinned = BLANK;
        self.checks = BLANK;

        let opposite_color = !self.side_to_move;
        let king_square = self.get_king_square(self.side_to_move);

        let bishops_and_queens = self.get_piece_type_mask(PieceType::Bishop)
            | self.get_piece_type_mask(PieceType::Queen);
        let rooks_and_queens =
            self.get_piece_type_mask(PieceType::Rook) | self.get_piece_type_mask(PieceType::Queen);
        let pinners = self.get_color_mask(opposite_color)
            & (BISHOP.get_moves(king_square) & bishops_and_queens
                | ROOK.get_moves(king_square) & rooks_and_queens);
        for pinner_square in pinners {
            let between =
                self.get_combined_mask() & BETWEEN.get(king_square, pinner_square).unwrap();
            if between == BLANK {
                self.checks |= BitBoard::from_square(pinner_square);
            } else if between.count_ones() == 1 {
                self.pinned |= between;
            }
        }

        self.checks |= self.get_color_mask(opposite_color)
            & KNIGHT.get_moves(king_square)
            & self.get_piece_type_mask(PieceType::Knight);

        self.checks |= {
            let mut all_pawn_attacks = BLANK;
            for attacked_square in
                self.get_color_mask(opposite_color) & self.get_piece_type_mask(PieceType::Pawn)
            {
                all_pawn_attacks |= PAWN.get_captures(attacked_square, opposite_color);
            }
            all_pawn_attacks & BitBoard::from_square(king_square)
        };

        self.checks |= self.get_color_mask(opposite_color)
            & KING.get_moves(king_square)
            & self.get_piece_type_mask(PieceType::King);

        self
    }

    fn update_legal_moves(&mut self) -> &mut Self {
        let mut moves = LegalMoves::new();
        let color_mask = self.get_color_mask(self.side_to_move);

        for i in 0..NUMBER_PIECE_TYPES {
            let piece_type = PieceType::from_index(i).unwrap();
            let free_pieces_mask =
                color_mask & self.get_piece_type_mask(piece_type) & !self.get_pin_mask();

            for square in free_pieces_mask {
                let mut full = match piece_type {
                    PieceType::Pawn => {
                        (PAWN.get_moves(square, self.side_to_move) & !self.combined_mask)
                            | (PAWN.get_captures(square, self.side_to_move)
                                & self.get_color_mask(!self.side_to_move))
                    }
                    PieceType::Knight => KNIGHT.get_moves(square) & !color_mask,
                    PieceType::King => KING.get_moves(square) & !color_mask,
                    PieceType::Bishop => BISHOP.get_moves(square),
                    PieceType::Rook => ROOK.get_moves(square),
                    PieceType::Queen => QUEEN.get_moves(square),
                };

                match piece_type {
                    PieceType::Pawn | PieceType::Knight | PieceType::King => {}
                    _ => {
                        let mut legals = BLANK;
                        for destination in full {
                            let destination_mask = BitBoard::from_square(destination);
                            let between_mask = BETWEEN.get(square, destination).unwrap();

                            match ((between_mask | destination_mask) & self.combined_mask)
                                .count_ones()
                            {
                                0 => {
                                    legals |= destination_mask;
                                }
                                1 => match self.get_piece_color_on(destination) {
                                    Some(c) => {
                                        if c == !self.side_to_move {
                                            legals |= destination_mask;
                                        }
                                    }
                                    None => {}
                                },
                                _ => {}
                            }
                        }
                        full = legals;
                    }
                }

                for one in full
                    .into_iter()
                    .map(|s| mv!(piece_type, square, s))
                    .filter(|m| {
                        self.clone()
                            .apply_move_unchecked(*m)
                            .update_pins_and_checks()
                            .get_check_mask()
                            .count_ones()
                            == 0
                    })
                {
                    if (one.get_piece_type() == PieceType::Pawn) & {
                        let destination_rank = one.get_destination_square().get_rank();
                        match self.side_to_move {
                            Color::White => destination_rank == Rank::Eighth,
                            Color::Black => destination_rank == Rank::First,
                        }
                    } {
                        // Generate promotion moves
                        let s = one.get_source_square();
                        let d = one.get_destination_square();
                        moves.insert(mv!(PieceType::Pawn, s, d, PromotionPieceType::Knight));
                        moves.insert(mv!(PieceType::Pawn, s, d, PromotionPieceType::Bishop));
                        moves.insert(mv!(PieceType::Pawn, s, d, PromotionPieceType::Rook));
                        moves.insert(mv!(PieceType::Pawn, s, d, PromotionPieceType::Queen));
                    } else {
                        moves.insert(one);
                    }
                }

                // TODO castling processing
                // TODO en passant processing
            }
        }

        self.legal_moves = moves;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use unindent::unindent;

    #[test]
    fn create_from_string() {
        assert_eq!(
            format!(
                "{}",
                BoardBuilder::try_from(&ChessBoard::default()).unwrap()
            ),
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        );
    }

    #[test]
    fn square_emptiness() {
        let board = ChessBoard::default();
        let a1 = Square::A1;
        let a3 = Square::A3;
        assert_eq!(board.is_empty_square(a1), false);
        assert_eq!(board.is_empty_square(a3), true);
    }

    #[rustfmt::skip]
    #[test]
    fn display_representation() {
        let board = ChessBoard::default();
        let board_str = 
        "   white  KQkq
            ╔════════════════════════╗
         8  ║ r  n  b  q  k  b  n  r ║
         7  ║ p  p  p  p  p  p  p  p ║
         6  ║                        ║
         5  ║                        ║
         4  ║                        ║
         3  ║                        ║
         2  ║ P  P  P  P  P  P  P  P ║
         1  ║ R  N  B  Q  K  B  N  R ║
            ╚════════════════════════╝
              a  b  c  d  e  f  g  h
        ";
        println!("{}", board);
        assert_eq!(
            format!("{}", board)
                .replace("\u{1b}[47;30m", "")
                .replace("\u{1b}[47m", "")
                .replace("\u{1b}[0m", ""),
            unindent(board_str)
        );

        let mut board = ChessBoard::default();
        board.set_flipped_view(true);
        let board_str =         
        "   white  KQkq
            ╔════════════════════════╗
         1  ║ R  N  B  K  Q  B  N  R ║
         2  ║ P  P  P  P  P  P  P  P ║
         3  ║                        ║
         4  ║                        ║
         5  ║                        ║
         6  ║                        ║
         7  ║ p  p  p  p  p  p  p  p ║
         8  ║ r  n  b  k  q  b  n  r ║
            ╚════════════════════════╝
              h  g  f  e  d  c  b  a
        ";
        println!("{}", board);
        assert_eq!(
            format!("{}", board)
                .replace("\u{1b}[47;30m", "")
                .replace("\u{1b}[47m", "")
                .replace("\u{1b}[0m", ""),
            unindent(board_str)
        );
    }

    #[test]
    fn kings_position() {
        let color = Color::White;
        assert_eq!(ChessBoard::default().get_king_square(color), Square::E1);
    }

    #[rustfmt::skip]
    #[test]
    fn masks() {
        let board = ChessBoard::default();
        let combined_str = 
            "X X X X X X X X 
             X X X X X X X X 
             . . . . . . . . 
             . . . . . . . . 
             . . . . . . . . 
             . . . . . . . . 
             X X X X X X X X 
             X X X X X X X X 
            ";
        assert_eq!(
            format!("{}", board.get_combined_mask()),
            unindent(combined_str)
        );

        let white = Color::White;
        let whites_str = 
            ". . . . . . . . 
             . . . . . . . . 
             . . . . . . . . 
             . . . . . . . . 
             . . . . . . . . 
             . . . . . . . . 
             X X X X X X X X 
             X X X X X X X X 
            ";
        assert_eq!(
            format!("{}", board.get_color_mask(white)),
            unindent(whites_str)
        );

        let black = Color::Black;
        let blacks_str = 
            "X X X X X X X X 
             X X X X X X X X 
             . . . . . . . . 
             . . . . . . . . 
             . . . . . . . . 
             . . . . . . . . 
             . . . . . . . . 
             . . . . . . . . 
            ";
        assert_eq!(
            format!("{}", board.get_color_mask(black)),
            unindent(blacks_str)
        );
    }

    #[test]
    fn hash_comparison_for_different_boards() {
        let board = ChessBoard::default();
        assert_eq!(board.get_hash(), board.get_hash());

        let mut another_board = ChessBoard::default();
        another_board = another_board
            .make_move(mv!(PieceType::Pawn, Square::E2, Square::E4))
            .unwrap();
        assert_ne!(board.get_hash(), another_board.get_hash());
    }

    #[test]
    fn checks_and_pins() {
        let board =
            ChessBoard::from_str("rnbqkbnr/pppppppp/8/8/8/5N2/PPPPPPPP/RNBQKB1R w KQkq - 0 1")
                .unwrap();
        let checkers: Vec<Square> = board.get_check_mask().into_iter().collect();
        assert_eq!(checkers, vec![]);

        let board = ChessBoard::from_str("8/8/5k2/8/3Q2N1/5K2/8/8 b - - 0 1").unwrap();
        let checkers: Vec<Square> = board.get_check_mask().into_iter().collect();
        assert_eq!(checkers, vec![Square::D4, Square::G4]);

        let board = ChessBoard::from_str("8/8/5k2/4p3/8/2Q2K2/8/8 b - - 0 1").unwrap();
        let pinned = board.get_pin_mask().to_square();
        assert_eq!(pinned, Square::E5);
    }

    #[test]
    fn board_builded_from_fen_validation() {
        assert!(ChessBoard::from_str("8/8/5k2/8/5Q2/5K2/8/8 w - - 0 1").is_err());
        assert!(ChessBoard::from_str("8/8/5k2/8/5Q2/5K2/8/8 w KQkq - 0 1").is_err());
        assert!(ChessBoard::from_str("8/8/5k2/8/5Q2/5K2/8/8 w - f5 0 1").is_err());
        assert!(ChessBoard::from_str("k7/K7/8/8/8/8/8/8 b - - 0 1").is_err());
        assert!(ChessBoard::from_str(
            "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 1"
        )
        .is_ok());
    }

    #[test]
    fn legal_moves_number_equality() {
        assert_eq!(ChessBoard::default().get_legal_moves().len(), 20);
        assert_eq!(
            ChessBoard::from_str("Q2k4/8/3K4/8/8/8/8/8 b - - 0 1")
                .unwrap()
                .get_legal_moves()
                .len(),
            0
        );
        assert_eq!(
            ChessBoard::from_str("rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 1")
                .unwrap()
                .get_legal_moves()
                .len(),
            29
        );
        assert_eq!(
            ChessBoard::from_str("3k4/3P4/3K4/8/8/8/8/8 b - - 0 1")
                .unwrap()
                .get_legal_moves()
                .len(),
            0
        );
    }

    #[test]
    fn promotion() {
        let board = ChessBoard::from_str("1r5k/P7/7K/8/8/8/8/8 w - - 0 1").unwrap();
        for one in board.get_legal_moves() {
            println!("{}", one);
        }

        assert_eq!(board.get_legal_moves().len(), 11);
    }
}
