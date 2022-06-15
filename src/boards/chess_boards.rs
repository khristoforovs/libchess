//! Chess Board module
//!
//! This module defines the representation of position on the board
//! (including Zobrist hash calculation) Implements the logics of
//! moving pieces and inferring the board status

use crate::boards::{
    squares, BitBoard, BoardBuilder, BoardMove, BoardMoveOption, DisplayAmbiguityType, File,
    PieceMove, PositionHashValueType, PromotionPieceType, Rank, Square, BLANK, FILES, RANKS,
    SQUARES_NUMBER, ZOBRIST_TABLES as ZOBRIST,
};
use crate::errors::ChessBoardError as Error;
use crate::move_masks::{
    BETWEEN_TABLE as BETWEEN, BISHOP_TABLE as BISHOP, KING_TABLE as KING, KNIGHT_TABLE as KNIGHT,
    PAWN_TABLE as PAWN, QUEEN_TABLE as QUEEN, ROOK_TABLE as ROOK,
};
use crate::{castle_king_side, castle_queen_side, mv};
use crate::{CastlingRights, Color, Piece, PieceType, COLORS_NUMBER, PIECE_TYPES_NUMBER};
use colored::Colorize;
use either::Either;
use std::collections::hash_set::HashSet;
use std::fmt;
use std::str::FromStr;

pub type LegalMoves = HashSet<BoardMove>;

/// Represents the board status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardStatus {
    Ongoing,
    CheckMated(Color),
    TheoreticalDrawDeclared,
    Stalemate,
}

/// The Chess Board. No more, no less
///
/// Represents any available board position. Can be initialized by the FEN-
/// string (most recommended) or directly from a BoardBuilder struct. Checks
/// the sanity of the position, so if the struct is created the position is valid.
/// If the initial position is not the terminal (stalemate or checkmate),
/// you can generate another valid board after calling .make_move(&self, next_move: ChessMove)
/// (of course the move should be legal).
///
/// Also it implements the board visualization (in terminal)
///
/// ## Examples
/// ```
/// use libchess::boards::{ChessBoard, BoardMove, BoardMoveOption, PieceMove, squares::*};
/// use libchess::{castle_king_side, castle_queen_side, mv};
/// use libchess::PieceType::*;
/// use std::str::FromStr;
///
/// println!("{}", ChessBoard::default());
///
/// let board = ChessBoard::from_str("8/P5k1/2b3p1/5p2/5K2/7R/8/8 w - - 13 61").unwrap();
/// println!("{}", board);
/// println!("{}", board.as_fen());
/// println!("{}", board.make_move(mv!(King, F4, G5)).unwrap());
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ChessBoard {
    pieces_mask: [BitBoard; PIECE_TYPES_NUMBER],
    colors_mask: [BitBoard; COLORS_NUMBER],
    combined_mask: BitBoard,
    side_to_move: Color,
    castle_rights: [CastlingRights; COLORS_NUMBER],
    en_passant: Option<Square>,
    pinned: BitBoard,
    checks: BitBoard,
    flipped_view: bool,
    is_terminal_position: bool,
    hash: PositionHashValueType,
}

impl TryFrom<&BoardBuilder> for ChessBoard {
    type Error = Error;

    fn try_from(builder: &BoardBuilder) -> Result<Self, Self::Error> {
        let mut board = ChessBoard::new();

        for i in 0..SQUARES_NUMBER {
            let square = Square::new(i as u8).unwrap();
            if let Some(piece) = builder[square] {
                board.put_piece(piece, square);
            }
        }

        board
            .set_side_to_move(builder.get_side_to_move())
            .set_en_passant(builder.get_en_passant())
            .set_castling_rights(Color::White, builder.get_castle_rights(Color::White))
            .set_castling_rights(Color::Black, builder.get_castle_rights(Color::Black))
            .update_pins_and_checks()
            .update_terminal_status();

        board.hash = ZOBRIST.calculate_position_hash(&board);

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
        BoardBuilder::from_str(value)?.try_into()
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
    fn new() -> Self {
        ChessBoard {
            pieces_mask: [BLANK; PIECE_TYPES_NUMBER],
            colors_mask: [BLANK; COLORS_NUMBER],
            combined_mask: BLANK,
            side_to_move: Color::White,
            castle_rights: [CastlingRights::BothSides; COLORS_NUMBER],
            en_passant: None,
            pinned: BLANK,
            checks: BLANK,
            flipped_view: false,
            is_terminal_position: false,
            hash: 0,
        }
    }

    pub fn validate(&self) -> Option<Error> {
        // make sure that is no color overlapping
        if self.get_color_mask(Color::White) & self.get_color_mask(Color::Black) != BLANK {
            return Some(Error::InvalidPositionColorsOverlap);
        };

        // check overlapping of piece type masks
        for i in 0..(PIECE_TYPES_NUMBER - 1) {
            for j in i + 1..PIECE_TYPES_NUMBER {
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
            (0..PIECE_TYPES_NUMBER).fold(BLANK, |current, i| {
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
        let white_rook_mask =
            self.get_piece_type_mask(PieceType::Rook) & self.get_color_mask(Color::White);
        if self.get_king_square(Color::White) == squares::E1 {
            let validation_mask = match self.get_castle_rights(Color::White) {
                CastlingRights::Neither => BLANK,
                CastlingRights::QueenSide => BitBoard::from_square(squares::A1),
                CastlingRights::KingSide => BitBoard::from_square(squares::H1),
                CastlingRights::BothSides => {
                    BitBoard::from_square(squares::A1) | BitBoard::from_square(squares::H1)
                }
            };
            if (white_rook_mask & validation_mask).count_ones() != validation_mask.count_ones() {
                return Some(Error::InvalidBoardInconsistentCastlingRights);
            }
        } else {
            match self.get_castle_rights(Color::White) {
                CastlingRights::Neither => {}
                _ => {
                    return Some(Error::InvalidBoardInconsistentCastlingRights);
                }
            }
        }

        let black_rook_mask =
            self.get_piece_type_mask(PieceType::Rook) & self.get_color_mask(Color::Black);
        if self.get_king_square(Color::Black) == squares::E8 {
            let validation_mask = match self.get_castle_rights(Color::Black) {
                CastlingRights::Neither => BLANK,
                CastlingRights::QueenSide => BitBoard::from_square(squares::A8),
                CastlingRights::KingSide => BitBoard::from_square(squares::H8),
                CastlingRights::BothSides => {
                    BitBoard::from_square(squares::A8) | BitBoard::from_square(squares::H8)
                }
            };
            if (black_rook_mask & validation_mask).count_ones() != validation_mask.count_ones() {
                return Some(Error::InvalidBoardInconsistentCastlingRights);
            }
        } else {
            match self.get_castle_rights(Color::Black) {
                CastlingRights::Neither => {}
                _ => {
                    return Some(Error::InvalidBoardInconsistentCastlingRights);
                }
            }
        }

        None
    }

    /// Returns a FEN string of current position
    #[inline]
    pub fn as_fen(&self) -> String {
        format!("{}", BoardBuilder::from_board(self, 0, 1))
    }

    /// Returns a Bitboard mask of same-color pieces
    #[inline]
    pub fn get_color_mask(&self, color: Color) -> BitBoard {
        self.colors_mask[color.to_index()]
    }

    /// Returns a Bitboard mask for all pieces on the board
    #[inline]
    pub fn get_combined_mask(&self) -> BitBoard {
        self.combined_mask
    }

    /// Returns a square for king-piece of specified color
    #[inline]
    pub fn get_king_square(&self, color: Color) -> Square {
        (self.get_piece_type_mask(PieceType::King) & self.get_color_mask(color)).to_square()
    }

    /// Returns a Bitboard mask for all pieces of the same  specified type
    #[inline]
    pub fn get_piece_type_mask(&self, piece_type: PieceType) -> BitBoard {
        self.pieces_mask[piece_type.to_index()]
    }

    /// Returns a Bitboard mask for all pieces which pins the king with
    /// color defined by ``board.get_side_to_move()``
    #[inline]
    pub fn get_pin_mask(&self) -> BitBoard {
        self.pinned
    }

    /// Returns the castling rights for specified color.
    ///
    /// The presence of castling rights does not mean that king can castle at
    /// this move (checks, extra pieces on backrank, etc.).
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

    /// Returns a Bitboard mask for all pieces which check the king with
    /// color defined by ``board.get_side_to_move()``
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

    /// Returns true if enabled the option of flipped print
    #[inline]
    pub fn get_print_flipped(&mut self) -> bool {
        self.flipped_view
    }

    /// Sets the flipped view for the visualization via ``fmt::Display``
    #[inline]
    pub fn set_print_flipped(&mut self, flipped: bool) {
        self.flipped_view = flipped
    }

    /// Returns a PieceType object if the square is not empty, None otherwise
    pub fn get_piece_type_on(&self, square: Square) -> Option<PieceType> {
        let bitboard = BitBoard::from_square(square);
        if self.get_combined_mask() & bitboard == BLANK {
            None
        } else if (self.get_piece_type_mask(PieceType::Pawn)
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
        } else if self.get_piece_type_mask(PieceType::Rook) & bitboard != BLANK {
            Some(PieceType::Rook)
        } else if self.get_piece_type_mask(PieceType::Queen) & bitboard != BLANK {
            Some(PieceType::Queen)
        } else {
            Some(PieceType::King)
        }
    }

    /// Returns a Color object if the square is not empty, None otherwise
    pub fn get_piece_color_on(&self, square: Square) -> Option<Color> {
        if (self.get_color_mask(Color::White) & BitBoard::from_square(square)) != BLANK {
            Some(Color::White)
        } else if (self.get_color_mask(Color::Black) & BitBoard::from_square(square)) != BLANK {
            Some(Color::Black)
        } else {
            None
        }
    }

    /// Returns true if specified move is legal for current position
    pub fn is_legal_move(&self, chess_move: BoardMove) -> bool {
        match chess_move.get_move_option() {
            BoardMoveOption::MovePiece(m) => {
                // Check source square
                let source_square = m.get_source_square();
                if (self.get_piece_type_mask(m.get_piece_type())
                    & self.get_color_mask(self.side_to_move)
                    & BitBoard::from_square(source_square))
                .count_ones()
                    != 1
                {
                    return false;
                }

                // Check destination square availability
                let destination_square = m.get_destination_square();
                let destination_mask = match m.get_piece_type() {
                    PieceType::Pawn => {
                        PAWN.get_moves(source_square, self.side_to_move)
                            & !self.get_color_mask(self.side_to_move)
                            | PAWN.get_captures(source_square, self.side_to_move)
                                & self.get_color_mask(!self.side_to_move)
                    }
                    PieceType::Knight => {
                        KNIGHT.get_moves(source_square) & !self.get_color_mask(self.side_to_move)
                    }
                    PieceType::Bishop => {
                        let between = BETWEEN.get(source_square, destination_square);
                        if between.is_none()
                            | ((between.unwrap() & self.get_combined_mask()).count_ones() > 0)
                        {
                            return false;
                        }
                        BISHOP.get_moves(source_square) & !self.get_color_mask(self.side_to_move)
                    }
                    PieceType::Rook => {
                        let between = BETWEEN.get(source_square, destination_square);
                        if between.is_none()
                            | ((between.unwrap() & self.get_combined_mask()).count_ones() > 0)
                        {
                            return false;
                        }
                        ROOK.get_moves(source_square) & !self.get_color_mask(self.side_to_move)
                    }
                    PieceType::Queen => {
                        let between = BETWEEN.get(source_square, destination_square);
                        if between.is_none()
                            | ((between.unwrap() & self.get_combined_mask()).count_ones() > 0)
                        {
                            return false;
                        }
                        QUEEN.get_moves(source_square) & !self.get_color_mask(self.side_to_move)
                    }
                    PieceType::King => {
                        KING.get_moves(source_square) & !self.get_color_mask(self.side_to_move)
                    }
                };
                if (destination_mask & BitBoard::from_square(destination_square)).count_ones() != 1
                {
                    return false;
                }

                // Check promotions
                if (m.get_promotion().is_some())
                    & (m.get_piece_type() != PieceType::Pawn)
                    & (destination_square.get_rank() != self.side_to_move.get_back_rank())
                {
                    return false;
                }

                // Checks
                if self
                    .clone()
                    .move_piece(m)
                    .update_pins_and_checks()
                    .get_check_mask()
                    .count_ones()
                    != 0
                {
                    return false;
                }
            }
            BoardMoveOption::CastleKingSide => {
                let is_not_check = self.get_check_mask().count_ones() == 0;
                if !self.get_castle_rights(self.side_to_move).has_kingside() {
                    return false;
                }
                let (square_king_side_1, square_king_side_2) = match self.side_to_move {
                    Color::White => (squares::F1, squares::G1),
                    Color::Black => (squares::F8, squares::G8),
                };
                let is_king_side_under_attack = self.is_under_attack(square_king_side_1)
                    | self.is_under_attack(square_king_side_2);
                let king_side_between_mask = match self.side_to_move {
                    Color::White => BETWEEN.get(squares::E1, squares::H1).unwrap(),
                    Color::Black => BETWEEN.get(squares::E8, squares::H8).unwrap(),
                };
                let is_empty_king_side =
                    (king_side_between_mask & self.get_combined_mask()).count_ones() == 0;
                if !(is_not_check & !is_king_side_under_attack & is_empty_king_side) {
                    return false;
                }
            }
            BoardMoveOption::CastleQueenSide => {
                let is_not_check = self.get_check_mask().count_ones() == 0;
                if !self.get_castle_rights(self.side_to_move).has_queenside() {
                    return false;
                }

                let (square_queen_side_1, square_queen_side_2) = match self.side_to_move {
                    Color::White => (squares::D1, squares::C1),
                    Color::Black => (squares::D8, squares::C8),
                };
                let is_queen_side_under_attack = self.is_under_attack(square_queen_side_1)
                    | self.is_under_attack(square_queen_side_2);
                let queen_side_between_mask = match self.side_to_move {
                    Color::White => BETWEEN.get(squares::E1, squares::A1).unwrap(),
                    Color::Black => BETWEEN.get(squares::E8, squares::A8).unwrap(),
                };
                let is_empty_queen_side =
                    (queen_side_between_mask & self.get_combined_mask()).count_ones() == 0;
                if !(is_not_check & !is_queen_side_under_attack & is_empty_queen_side) {
                    return false;
                }
            }
        }

        true
    }

    /// Returns true if current side has at least one legal move
    pub fn is_terminal(&self) -> bool {
        self.is_terminal_position
    }

    /// Returns a HashSet of all legal moves for current board
    pub fn get_legal_moves(&self) -> LegalMoves {
        let mut moves = LegalMoves::new();
        let color_mask = self.get_color_mask(self.side_to_move);
        let en_passant_mask = match self.get_en_passant() {
            Some(sq) => BitBoard::from_square(sq),
            None => BLANK,
        };

        for i in 0..PIECE_TYPES_NUMBER {
            let piece_type = PieceType::from_index(i).unwrap();
            let free_pieces_mask = color_mask & self.get_piece_type_mask(piece_type);

            for square in free_pieces_mask {
                let mut full = match piece_type {
                    PieceType::Pawn => {
                        (PAWN.get_moves(square, self.side_to_move) & !self.combined_mask)
                            | (PAWN.get_captures(square, self.side_to_move)
                                & (self.get_color_mask(!self.side_to_move) | en_passant_mask))
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
                            .move_piece(m.piece_move().unwrap())
                            .update_pins_and_checks()
                            .get_check_mask()
                            .count_ones()
                            == 0
                    })
                {
                    let m = one.piece_move().unwrap();
                    if (m.get_piece_type() == PieceType::Pawn) & {
                        let destination_rank = m.get_destination_square().get_rank();
                        match self.side_to_move {
                            Color::White => destination_rank == Rank::Eighth,
                            Color::Black => destination_rank == Rank::First,
                        }
                    } {
                        // Generate promotion moves
                        let s = m.get_source_square();
                        let d = m.get_destination_square();
                        moves.insert(mv!(PieceType::Pawn, s, d, PromotionPieceType::Knight));
                        moves.insert(mv!(PieceType::Pawn, s, d, PromotionPieceType::Bishop));
                        moves.insert(mv!(PieceType::Pawn, s, d, PromotionPieceType::Rook));
                        moves.insert(mv!(PieceType::Pawn, s, d, PromotionPieceType::Queen));
                    } else {
                        moves.insert(one);
                    }
                }
            }
        }

        // Check if castling is legal
        let is_not_check = self.get_check_mask().count_ones() == 0;
        if self.get_castle_rights(self.side_to_move).has_kingside() {
            let (square_king_side_1, square_king_side_2) = match self.side_to_move {
                Color::White => (squares::F1, squares::G1),
                Color::Black => (squares::F8, squares::G8),
            };
            let is_king_side_under_attack =
                self.is_under_attack(square_king_side_1) | self.is_under_attack(square_king_side_2);
            let king_side_between_mask = match self.side_to_move {
                Color::White => BETWEEN.get(squares::E1, squares::H1).unwrap(),
                Color::Black => BETWEEN.get(squares::E8, squares::H8).unwrap(),
            };
            let is_empty_king_side =
                (king_side_between_mask & self.get_combined_mask()).count_ones() == 0;
            if is_not_check & !is_king_side_under_attack & is_empty_king_side {
                moves.insert(castle_king_side!());
            }
        }

        if self.get_castle_rights(self.side_to_move).has_queenside() {
            let (square_queen_side_1, square_queen_side_2) = match self.side_to_move {
                Color::White => (squares::D1, squares::C1),
                Color::Black => (squares::D8, squares::C8),
            };
            let is_queen_side_under_attack = self.is_under_attack(square_queen_side_1)
                | self.is_under_attack(square_queen_side_2);
            let queen_side_between_mask = match self.side_to_move {
                Color::White => BETWEEN.get(squares::E1, squares::A1).unwrap(),
                Color::Black => BETWEEN.get(squares::E8, squares::A8).unwrap(),
            };
            let is_empty_queen_side =
                (queen_side_between_mask & self.get_combined_mask()).count_ones() == 0;
            if is_not_check & !is_queen_side_under_attack & is_empty_queen_side {
                moves.insert(castle_queen_side!());
            }
        }

        moves
    }

    /// Returns the hash of the position. Is used to detect the repetition draw
    pub fn get_hash(&self) -> PositionHashValueType {
        self.hash
    }

    /// Returns position status on the board
    pub fn get_status(&self) -> BoardStatus {
        if self.is_terminal_position {
            if self.checks.count_ones() > 0 {
                BoardStatus::CheckMated(self.side_to_move)
            } else {
                BoardStatus::Stalemate
            }
        } else if self.is_theoretical_draw_on_board() {
            BoardStatus::TheoreticalDrawDeclared
        } else {
            BoardStatus::Ongoing
        }
    }

    /// Returns true if neither white and black can not checkmate each other
    pub fn is_theoretical_draw_on_board(&self) -> bool {
        let white_pieces_number = self.get_color_mask(Color::White).count_ones();
        let black_pieces_number = self.get_color_mask(Color::Black).count_ones();

        if (white_pieces_number > 2) | (black_pieces_number > 2) {
            return false;
        }

        let bishops_and_knights = self.get_piece_type_mask(PieceType::Knight)
            | self.get_piece_type_mask(PieceType::Bishop);

        let white_can_not_checkmate = match white_pieces_number {
            1 => true,
            2 => self.get_color_mask(Color::White) & bishops_and_knights != BLANK,
            _ => unreachable!(),
        };
        let black_can_not_checkmate = match black_pieces_number {
            1 => true,
            2 => self.get_color_mask(Color::Black) & bishops_and_knights != BLANK,
            _ => unreachable!(),
        };

        white_can_not_checkmate & black_can_not_checkmate
    }

    /// This method is needed to represent the chess move without any ambiguity in PGN-like strings
    pub fn get_move_ambiguity_type(
        &self,
        piece_move: PieceMove,
    ) -> Result<DisplayAmbiguityType, Error> {
        if !self.is_legal_move(BoardMove::new(BoardMoveOption::MovePiece(piece_move))) {
            return Err(Error::IllegalMoveDetected);
        }

        let piece_type = piece_move.get_piece_type();
        let source_square = piece_move.get_source_square();
        let destination_square = piece_move.get_destination_square();

        if piece_type == PieceType::Pawn {
            if source_square.get_file() != destination_square.get_file() {
                return Ok(DisplayAmbiguityType::ExtraFile);
            }
        } else if piece_type == PieceType::King {
            return Ok(DisplayAmbiguityType::Neither);
        } else {
            let pieces_mask =
                self.get_piece_type_mask(piece_type) & self.get_color_mask(self.side_to_move);
            let piece_moves = match piece_type {
                PieceType::Knight => KNIGHT.get_moves(destination_square),
                PieceType::Bishop => BISHOP.get_moves(destination_square),
                PieceType::Rook => ROOK.get_moves(destination_square),
                PieceType::Queen => QUEEN.get_moves(destination_square),
                _ => BLANK,
            };

            let between_filter: Box<dyn Fn(&Square) -> bool> = match piece_type {
                PieceType::Knight => Box::new(|_: &Square| true),
                _ => Box::new(|x: &Square| {
                    (BETWEEN
                        .get(*x, piece_move.get_destination_square())
                        .unwrap()
                        & self.combined_mask)
                        .count_ones()
                        == 0
                }),
            };

            let candidates = (piece_moves & pieces_mask)
                .into_iter()
                .filter(between_filter);

            if candidates.count() > 1 {
                if (BitBoard::from_file(source_square.get_file()) & pieces_mask).count_ones() > 1 {
                    return Ok(DisplayAmbiguityType::ExtraSquare);
                } else {
                    return Ok(DisplayAmbiguityType::ExtraFile);
                }
            }
        }

        Ok(DisplayAmbiguityType::Neither)
    }

    /// The method which allows to make moves on the board. Returns a new board instance
    /// if the move is legal
    ///
    /// The simplest way to generate moves is by picking one from a set of available moves:
    /// ``board.get_legal_moves()`` or by simply creating a new move via macros: ``mv!()``,
    /// ``castle_king_side!()`` and ``castle_queen_side!()``
    ///
    /// ```
    /// use libchess::boards::{ChessBoard, BoardMove, BoardMoveOption, PieceMove, squares::*};
    /// use libchess::{castle_king_side, castle_queen_side, mv};
    /// use libchess::PieceType::*;
    ///
    /// let board = ChessBoard::default();
    /// let next_board = board.make_move(mv!(Pawn, E2, E4)).unwrap();
    /// println!("{}", next_board);
    /// ```
    pub fn make_move_mut(&mut self, next_move: BoardMove) -> Result<&mut Self, Error> {
        if !self.is_legal_move(next_move) {
            return Err(Error::IllegalMoveDetected);
        }

        match next_move.get_move_option() {
            BoardMoveOption::MovePiece(m) => {
                self.move_piece(m);
            }
            BoardMoveOption::CastleKingSide => {
                let king_rank = match self.side_to_move {
                    Color::White => Rank::First,
                    Color::Black => Rank::Eighth,
                };
                self.move_piece(PieceMove::new(
                    PieceType::King,
                    Square::from_rank_file(king_rank, File::E),
                    Square::from_rank_file(king_rank, File::G),
                    None,
                ));
                self.move_piece(PieceMove::new(
                    PieceType::Rook,
                    Square::from_rank_file(king_rank, File::H),
                    Square::from_rank_file(king_rank, File::F),
                    None,
                ));
            }
            BoardMoveOption::CastleQueenSide => {
                let king_rank = match self.side_to_move {
                    Color::White => Rank::First,
                    Color::Black => Rank::Eighth,
                };
                self.move_piece(PieceMove::new(
                    PieceType::King,
                    Square::from_rank_file(king_rank, File::E),
                    Square::from_rank_file(king_rank, File::C),
                    None,
                ));
                self.move_piece(PieceMove::new(
                    PieceType::Rook,
                    Square::from_rank_file(king_rank, File::A),
                    Square::from_rank_file(king_rank, File::D),
                    None,
                ));
            }
        }

        let new_side_to_move = !self.side_to_move;
        self.update_castling_rights(next_move)
            .set_side_to_move(new_side_to_move)
            .update_en_passant(next_move)
            .update_pins_and_checks()
            .update_terminal_status();

        Ok(self)
    }

    pub fn make_move(&self, next_move: BoardMove) -> Result<Self, Error> {
        let mut next_board = self.clone();
        next_board.make_move_mut(next_move)?;
        Ok(next_board)
    }

    fn move_piece(&mut self, piece_move: PieceMove) -> &mut Self {
        let color = self
            .get_piece_color_on(piece_move.get_source_square())
            .unwrap();
        self.clear_square(piece_move.get_source_square())
            .clear_square(piece_move.get_destination_square())
            .put_piece(
                Piece(piece_move.get_piece_type(), color),
                piece_move.get_destination_square(),
            )
    }

    fn set_side_to_move(&mut self, color: Color) -> &mut Self {
        if color != self.side_to_move {
            self.hash ^= ZOBRIST.get_black_to_move_value();
        }

        self.side_to_move = color;
        self
    }

    fn set_castling_rights(&mut self, color: Color, rights: CastlingRights) -> &mut Self {
        let current_rights = self.castle_rights[color.to_index()];
        if current_rights != rights {
            self.hash ^= ZOBRIST.get_castling_rights_value(current_rights, color);
            self.hash ^= ZOBRIST.get_castling_rights_value(rights, color);
        }

        self.castle_rights[color.to_index()] = rights;
        self
    }

    fn set_en_passant(&mut self, square: Option<Square>) -> &mut Self {
        let current_ep = self.get_en_passant();
        if let Some(sq) = current_ep {
            self.hash ^= ZOBRIST.get_en_passant_value(sq);
        }
        if let Some(sq) = square {
            self.hash ^= ZOBRIST.get_en_passant_value(sq);
        }

        self.en_passant = square;
        self
    }

    fn put_piece(&mut self, piece: Piece, square: Square) -> &mut Self {
        self.clear_square(square);
        let square_bitboard = BitBoard::from_square(square);
        self.combined_mask |= square_bitboard;
        self.pieces_mask[piece.0.to_index()] |= square_bitboard;
        self.colors_mask[piece.1.to_index()] |= square_bitboard;

        self.hash ^= ZOBRIST.get_piece_square_value(piece, square);
        self
    }

    fn clear_square(&mut self, square: Square) -> &mut Self {
        match self.get_piece_type_on(square) {
            Some(piece_type) => {
                let color = self.get_piece_color_on(square).unwrap();
                let mask = !BitBoard::from_square(square);

                self.combined_mask &= mask;
                self.pieces_mask[piece_type.to_index()] &= mask;
                self.colors_mask[color.to_index()] &= mask;

                self.hash ^= ZOBRIST.get_piece_square_value(Piece(piece_type, color), square);
            }
            None => {}
        }
        self
    }

    fn update_pins_and_checks(&mut self) -> &mut Self {
        let king_square = self.get_king_square(self.side_to_move);
        (self.pinned, self.checks) = self.get_pins_and_checks(king_square);
        self
    }

    fn update_en_passant(&mut self, last_move: BoardMove) -> &mut Self {
        match last_move.get_move_option() {
            BoardMoveOption::MovePiece(m) => {
                let source_rank_index = m.get_source_square().get_rank().to_index();
                let destination_rank_index = m.get_destination_square().get_rank().to_index();
                if (m.get_piece_type() == PieceType::Pawn)
                    & (source_rank_index.abs_diff(destination_rank_index) == 2)
                {
                    let en_passant_square = Square::from_rank_file(
                        Rank::from_index((source_rank_index + destination_rank_index) / 2).unwrap(),
                        m.get_destination_square().get_file(),
                    );
                    self.set_en_passant(Some(en_passant_square));
                } else {
                    self.set_en_passant(None);
                }
            }
            _ => {
                self.set_en_passant(None);
            }
        }
        self
    }

    fn update_castling_rights(&mut self, last_move: BoardMove) -> &mut Self {
        self.set_castling_rights(
            self.side_to_move,
            self.get_castle_rights(self.side_to_move)
                - match last_move.get_move_option() {
                    BoardMoveOption::MovePiece(m) => match m.get_piece_type() {
                        PieceType::Rook => match m.get_source_square().get_file() {
                            File::H => CastlingRights::KingSide,
                            File::A => CastlingRights::QueenSide,
                            _ => CastlingRights::Neither,
                        },
                        PieceType::King => CastlingRights::BothSides,
                        _ => CastlingRights::Neither,
                    },
                    _ => CastlingRights::BothSides,
                },
        );
        self
    }

    fn update_terminal_status(&mut self) -> &mut Self {
        let color_mask = self.get_color_mask(self.side_to_move);
        let en_passant_mask = match self.get_en_passant() {
            Some(sq) => BitBoard::from_square(sq),
            None => BLANK,
        };

        for i in 0..PIECE_TYPES_NUMBER {
            let piece_type = PieceType::from_index(i).unwrap();
            let free_pieces_mask =
                color_mask & self.get_piece_type_mask(piece_type) & !self.get_pin_mask();

            for square in free_pieces_mask {
                let mut full = match piece_type {
                    PieceType::Pawn => {
                        (PAWN.get_moves(square, self.side_to_move) & !self.combined_mask)
                            | (PAWN.get_captures(square, self.side_to_move)
                                & (self.get_color_mask(!self.side_to_move) | en_passant_mask))
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

                if full
                    .into_iter()
                    .map(|s| mv!(piece_type, square, s))
                    .filter(|m| {
                        self.clone()
                            .move_piece(m.piece_move().unwrap())
                            .update_pins_and_checks()
                            .get_check_mask()
                            .count_ones()
                            == 0
                    })
                    .count()
                    > 0
                {
                    self.is_terminal_position = false;
                    return self;
                }
            }
        }

        // Check if castling is legal
        let is_not_check = self.get_check_mask().count_ones() == 0;
        if self.get_castle_rights(self.side_to_move).has_kingside() {
            let (square_king_side_1, square_king_side_2) = match self.side_to_move {
                Color::White => (squares::F1, squares::G1),
                Color::Black => (squares::F8, squares::G8),
            };
            let is_king_side_under_attack =
                self.is_under_attack(square_king_side_1) | self.is_under_attack(square_king_side_2);
            let king_side_between_mask = match self.side_to_move {
                Color::White => BETWEEN.get(squares::E1, squares::H1).unwrap(),
                Color::Black => BETWEEN.get(squares::E8, squares::H8).unwrap(),
            };
            let is_empty_king_side =
                (king_side_between_mask & self.get_combined_mask()).count_ones() == 0;
            if is_not_check & !is_king_side_under_attack & is_empty_king_side {
                self.is_terminal_position = false;
                return self;
            }
        }

        if self.get_castle_rights(self.side_to_move).has_queenside() {
            let (square_queen_side_1, square_queen_side_2) = match self.side_to_move {
                Color::White => (squares::D1, squares::C1),
                Color::Black => (squares::D8, squares::C8),
            };
            let is_queen_side_under_attack = self.is_under_attack(square_queen_side_1)
                | self.is_under_attack(square_queen_side_2);
            let queen_side_between_mask = match self.side_to_move {
                Color::White => BETWEEN.get(squares::E1, squares::A1).unwrap(),
                Color::Black => BETWEEN.get(squares::E8, squares::A8).unwrap(),
            };
            let is_empty_queen_side =
                (queen_side_between_mask & self.get_combined_mask()).count_ones() == 0;
            if is_not_check & !is_queen_side_under_attack & is_empty_queen_side {
                self.is_terminal_position = false;
                return self;
            }
        }

        self.is_terminal_position = true;
        self
    }

    fn get_pins_and_checks(&self, square: Square) -> (BitBoard, BitBoard) {
        let mut pinned = BLANK;
        let mut checks = BLANK;

        let opposite_color = !self.side_to_move;
        let bishops_and_queens = self.get_piece_type_mask(PieceType::Bishop)
            | self.get_piece_type_mask(PieceType::Queen);
        let rooks_and_queens =
            self.get_piece_type_mask(PieceType::Rook) | self.get_piece_type_mask(PieceType::Queen);

        let (bishop_mask, rook_mask) = (BISHOP.get_moves(square), ROOK.get_moves(square));
        let pinners = self.get_color_mask(opposite_color)
            & (bishop_mask & bishops_and_queens | rook_mask & rooks_and_queens);

        for pinner_square in pinners {
            let between = self.get_combined_mask() & BETWEEN.get(square, pinner_square).unwrap();
            if between == BLANK {
                checks |= BitBoard::from_square(pinner_square);
            } else if between.count_ones() == 1 {
                pinned |= between;
            }
        }

        checks |= self.get_color_mask(opposite_color)
            & KNIGHT.get_moves(square)
            & self.get_piece_type_mask(PieceType::Knight);

        checks |= {
            let mut all_pawn_attacks = BLANK;
            for attacked_square in
                self.get_color_mask(opposite_color) & self.get_piece_type_mask(PieceType::Pawn)
            {
                all_pawn_attacks |= PAWN.get_captures(attacked_square, opposite_color);
            }
            all_pawn_attacks & BitBoard::from_square(square)
        };

        checks |= self.get_color_mask(opposite_color)
            & KING.get_moves(square)
            & self.get_piece_type_mask(PieceType::King);

        (pinned, checks)
    }

    fn is_under_attack(&self, square: Square) -> bool {
        let (_, attacks) = self.get_pins_and_checks(square);
        attacks.count_ones() > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boards::{squares::*, BoardMove, BoardMoveOption, PieceMove, Square};
    use crate::PieceType::*;
    use unindent::unindent;

    #[test]
    fn create_from_string() {
        assert_eq!(
            format!("{}", BoardBuilder::from_board(&ChessBoard::default(), 0, 1)),
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        );
    }

    #[test]
    fn square_emptiness() {
        let board = ChessBoard::default();
        let a1 = A1;
        let a3 = A3;
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
        board.set_print_flipped(true);
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
        assert_eq!(ChessBoard::default().get_king_square(color), E1);
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
        another_board = another_board.make_move(mv!(Pawn, E2, E4)).unwrap();
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
        assert_eq!(checkers, vec![D4, G4]);

        let board = ChessBoard::from_str("8/8/5k2/4p3/8/2Q2K2/8/8 b - - 0 1").unwrap();
        let pinned = board.get_pin_mask().to_square();
        assert_eq!(pinned, E5);
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

        let board =
            ChessBoard::from_str("rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 1")
                .unwrap();
        for one in board.get_legal_moves() {
            assert_eq!(board.is_legal_move(one), true);
        }
    }

    #[test]
    fn promotion() {
        let board = ChessBoard::from_str("1r5k/P7/7K/8/8/8/8/8 w - - 0 1").unwrap();
        for one in board.get_legal_moves() {
            println!("{}", one);
        }

        assert_eq!(board.get_legal_moves().len(), 11);
    }

    #[test]
    fn en_passant() {
        let board =
            ChessBoard::from_str("rnbqkbnr/ppppppp1/8/4P2p/8/8/PPPP1PPP/PNBQKBNR b - - 0 1")
                .unwrap();

        let next_board = board.make_move(mv![Pawn, D7, D5]).unwrap();

        assert!(next_board.get_legal_moves().contains(&mv![Pawn, E5, D6]));
    }

    #[test]
    fn castling() {
        let board =
            ChessBoard::from_str("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1").unwrap();
        assert!(board.get_legal_moves().contains(&castle_king_side!()));
        assert!(board.get_legal_moves().contains(&castle_queen_side!()));

        let board =
            ChessBoard::from_str("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w kq - 0 1").unwrap();
        assert!(!board.get_legal_moves().contains(&castle_king_side!()));
        assert!(!board.get_legal_moves().contains(&castle_queen_side!()));

        let board =
            ChessBoard::from_str("r3k1nr/pp1ppppp/8/8/8/2Q5/PPPPPPPP/R3K2R b KQkq - 0 1").unwrap();
        assert!(!board.get_legal_moves().contains(&castle_king_side!()));
        assert!(!board.get_legal_moves().contains(&castle_queen_side!()));
    }

    #[test]
    fn kill_the_king() {
        assert!(ChessBoard::from_str("Q3k3/8/4K3/8/8/8/8/8 w - - 0 1").is_err());
    }
}
