//! Chess Board module
//!
//! This module defines the representation of position on the board
//! (including Zobrist hash calculation) Implements the logics of
//! moving pieces and inferring the board status

use crate::errors::LibChessError as Error;
use crate::move_masks::{
    BETWEEN_TABLE as BETWEEN, BISHOP_TABLE as BISHOP, KING_TABLE as KING, KNIGHT_TABLE as KNIGHT,
    PAWN_TABLE as PAWN, QUEEN_TABLE as QUEEN, RAYS_TABLE as RAYS, ROOK_TABLE as ROOK,
};
use crate::{
    castle_king_side, castle_queen_side, mv, squares, BitBoard, BoardBuilder, BoardMove,
    CastlingRights, Color, DisplayAmbiguityType, File, Piece, PieceMove, PieceType,
    PositionHashValueType, Rank, Square, BLANK, COLORS_NUMBER, FILES, PIECE_TYPES_NUMBER, RANKS,
    SQUARES_NUMBER, ZOBRIST_TABLES as ZOBRIST,
};
use crate::{Color::*, PieceType::*};
use colored::Colorize;
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
    FiftyMovesDrawDeclared,
    Stalemate,
}

/// The Chess Board. No more, no less
///
/// Represents any available board position. Can be initialized by the FEN-
/// string (most recommended) or directly from a BoardBuilder struct. Checks
/// the sanity of the position, so if the struct is created the position is valid.
/// If the initial position is not the terminal (stalemate or checkmate),
/// you can generate another valid board after calling .make_move(&self, next_move: ChessMove)
/// (of course, the move must be legal).
///
/// Also it implements the board visualization (in terminal)
///
/// ## Examples
/// ```
/// use libchess::PieceType::*;
/// use libchess::{castle_king_side, castle_queen_side, mv};
/// use libchess::{squares::*, BoardMove, ChessBoard, PieceMove};
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
    is_terminal_position: bool,
    moves_since_capture_or_pawn_move: usize,
    move_number: usize,
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
            .set_move_number(builder.get_move_number())
            .set_moves_since_capture_or_pawn_move(builder.get_moves_since_capture_or_pawn_move())
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

    fn try_from(fen: &mut BoardBuilder) -> Result<Self, Self::Error> { (&*fen).try_into() }
}

impl TryFrom<BoardBuilder> for ChessBoard {
    type Error = Error;

    fn try_from(fen: BoardBuilder) -> Result<Self, Self::Error> { (&fen).try_into() }
}

impl FromStr for ChessBoard {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        BoardBuilder::from_str(value)?.try_into()
    }
}

impl fmt::Display for ChessBoard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.render_straight()) }
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
            side_to_move: White,
            castle_rights: [CastlingRights::BothSides; COLORS_NUMBER],
            en_passant: None,
            pinned: BLANK,
            checks: BLANK,
            is_terminal_position: false,
            moves_since_capture_or_pawn_move: 0,
            move_number: 1,
            hash: 0,
        }
    }

    /// Validates the position on the board
    pub fn validate(&self) -> Option<Error> {
        use {squares::*, CastlingRights::*};

        // make sure that is no color overlapping
        if self.get_color_mask(White) & self.get_color_mask(Black) != BLANK {
            return Some(Error::InvalidPositionColorsOverlap);
        };

        // check overlapping of piece type masks
        for i in 0..(PIECE_TYPES_NUMBER - 1) {
            for j in (i + 1)..PIECE_TYPES_NUMBER {
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
        let king_mask = self.get_piece_type_mask(King);
        if (king_mask & self.get_color_mask(White)).count_ones() != 1 {
            return Some(Error::InvalidBoardMultipleOneColorKings);
        }
        if (king_mask & self.get_color_mask(White)).count_ones() != 1 {
            return Some(Error::InvalidBoardMultipleOneColorKings);
        }

        // make sure that opponent is not on check
        let mut cloned_board = *self;
        cloned_board.set_side_to_move(!self.side_to_move);
        cloned_board.update_pins_and_checks();
        if cloned_board.get_check_mask().count_ones() > 0 {
            return Some(Error::InvalidBoardOpponentIsOnCheck);
        }

        // validate en passant
        if let Some(square) = self.get_en_passant() {
            if self.get_piece_type_mask(Pawn)
                & self.get_color_mask(!self.side_to_move)
                & BitBoard::from_square(match !self.side_to_move {
                    White => square.up().unwrap(),
                    Black => square.down().unwrap(),
                })
                == BLANK
            {
                return Some(Error::InvalidBoardInconsistentEnPassant);
            }
        }

        // validate castling rights
        let white_rook_mask = self.get_piece_type_mask(Rook) & self.get_color_mask(White);
        if self.get_king_square(White) == E1 {
            let validation_mask = match self.get_castle_rights(White) {
                Neither => BLANK,
                QueenSide => BitBoard::from_square(A1),
                KingSide => BitBoard::from_square(H1),
                BothSides => BitBoard::from_square(A1) | BitBoard::from_square(H1),
            };
            if (white_rook_mask & validation_mask).count_ones() != validation_mask.count_ones() {
                return Some(Error::InvalidBoardInconsistentCastlingRights);
            }
        } else if self.get_castle_rights(White) != Neither {
            return Some(Error::InvalidBoardInconsistentCastlingRights);
        }

        let black_rook_mask = self.get_piece_type_mask(Rook) & self.get_color_mask(Black);
        if self.get_king_square(Black) == E8 {
            let validation_mask = match self.get_castle_rights(Black) {
                Neither => BLANK,
                QueenSide => BitBoard::from_square(A8),
                KingSide => BitBoard::from_square(H8),
                BothSides => BitBoard::from_square(A8) | BitBoard::from_square(H8),
            };
            if (black_rook_mask & validation_mask).count_ones() != validation_mask.count_ones() {
                return Some(Error::InvalidBoardInconsistentCastlingRights);
            }
        } else if self.get_castle_rights(Black) != Neither {
            return Some(Error::InvalidBoardInconsistentCastlingRights);
        }

        None
    }

    /// Unified method for rendering to terminal
    fn render<'a>(
        &self,
        ranks: impl Iterator<Item = &'a Rank>,
        files: impl Iterator<Item = &'a File> + Clone,
        footer: &str,
    ) -> String {
        let mut field_string = String::new();
        for rank in ranks {
            field_string = format!("{field_string}{}  ║", (rank).to_index() + 1);
            for file in files.clone() {
                let square = Square::from_rank_file(*rank, *file);
                field_string = if self.is_empty_square(square) {
                    if square.is_light() {
                        format!("{field_string}{}", "   ".on_white())
                    } else {
                        format!("{field_string}{}", "   ")
                    }
                } else {
                    let mut piece_type_str =
                        format!(" {} ", self.get_piece_type_on(square).unwrap());
                    piece_type_str = match self.get_piece_color_on(square).unwrap() {
                        White => piece_type_str.to_uppercase(),
                        Black => piece_type_str.to_lowercase(),
                    };

                    if square.is_light() {
                        format!("{field_string}{}", piece_type_str.black().on_white())
                    } else {
                        format!("{field_string}{piece_type_str}")
                    }
                }
            }
            field_string = format!("{field_string}║\n");
        }

        let board_string = format!(
            "   {}  {}{}\n{}\n{}{}\n{}\n",
            self.get_side_to_move(),
            format!("{}", self.get_castle_rights(White)).to_uppercase(),
            self.get_castle_rights(Black),
            "   ╔════════════════════════╗",
            field_string,
            "   ╚════════════════════════╝",
            footer,
        );
        board_string
    }

    /// Returns ASCII-representation of the board as a String
    pub fn render_straight(&self) -> String {
        let footer = "     a  b  c  d  e  f  g  h";
        self.render(RANKS.iter().rev(), FILES.iter(), footer)
    }

    /// Returns ASCII-representation of the flipped board as a String
    pub fn render_flipped(&self) -> String {
        let footer = "     h  g  f  e  d  c  b  a";
        self.render(RANKS.iter(), FILES.iter().rev(), footer)
    }

    /// Returns a FEN string of current position
    #[inline]
    pub fn as_fen(&self) -> String { format!("{}", BoardBuilder::from(*self)) }

    /// Returns a Bitboard mask of same-color pieces
    #[inline]
    pub fn get_color_mask(&self, color: Color) -> BitBoard { self.colors_mask[color.to_index()] }

    /// Returns a Bitboard mask for all pieces on the board
    #[inline]
    pub fn get_combined_mask(&self) -> BitBoard { self.combined_mask }

    /// Returns a square for king-piece of specified color
    #[inline]
    pub fn get_king_square(&self, color: Color) -> Square {
        (self.get_piece_type_mask(King) & self.get_color_mask(color)).to_square()
    }

    /// Returns a Bitboard mask for all pieces of the same  specified type
    #[inline]
    pub fn get_piece_type_mask(&self, piece_type: PieceType) -> BitBoard {
        self.pieces_mask[piece_type.to_index()]
    }

    /// Returns a Bitboard mask for all pieces which pins the king with
    /// color defined by ``board.get_side_to_move()``
    #[inline]
    pub fn get_pin_mask(&self) -> BitBoard { self.pinned }

    /// Returns the castling rights for specified color.
    ///
    /// The presence of castling rights does not mean that king can castle at
    /// this move (checks, extra pieces on backrank, etc.).
    #[inline]
    pub fn get_castle_rights(&self, color: Color) -> CastlingRights {
        self.castle_rights[color.to_index()]
    }

    #[inline]
    pub fn get_side_to_move(&self) -> Color { self.side_to_move }

    #[inline]
    pub fn get_en_passant(&self) -> Option<Square> { self.en_passant }

    /// Returns a move number
    #[inline]
    pub fn get_move_number(&self) -> usize { self.move_number }

    /// Returns a number of moves since last capture or pawn move (is used
    /// to determine the game termination by the 50-move rule)
    #[inline]
    pub fn get_moves_since_capture_or_pawn_move(&self) -> usize {
        self.moves_since_capture_or_pawn_move
    }

    /// Returns a Bitboard mask for all pieces which check the king with
    /// color defined by ``board.get_side_to_move()``
    #[inline]
    pub fn get_check_mask(&self) -> BitBoard { self.checks }

    #[inline]
    pub fn is_empty_square(&self, square: Square) -> bool {
        (self.combined_mask & BitBoard::from_square(square)).count_ones() == 0
    }

    /// Returns Some(PieceType) object if the square is not empty, None otherwise
    pub fn get_piece_type_on(&self, square: Square) -> Option<PieceType> {
        let bitboard = BitBoard::from_square(square);
        if self.get_combined_mask() & bitboard == BLANK {
            return None;
        }

        if (self.get_piece_type_mask(Pawn)
            | self.get_piece_type_mask(Knight)
            | self.get_piece_type_mask(Bishop))
            & bitboard
            != BLANK
        {
            if self.get_piece_type_mask(Pawn) & bitboard != BLANK {
                Some(Pawn)
            } else if self.get_piece_type_mask(Knight) & bitboard != BLANK {
                Some(Knight)
            } else {
                Some(Bishop)
            }
        } else {
            if self.get_piece_type_mask(Rook) & bitboard != BLANK {
                Some(Rook)
            } else if self.get_piece_type_mask(Queen) & bitboard != BLANK {
                Some(Queen)
            } else {
                Some(King)
            }
        }
    }

    /// Returns Some(Color) object if the square is not empty, None otherwise
    pub fn get_piece_color_on(&self, square: Square) -> Option<Color> {
        let bitboard = BitBoard::from_square(square);
        if self.get_combined_mask() & bitboard == BLANK {
            return None;
        }

        if (self.get_color_mask(White) & BitBoard::from_square(square)) != BLANK {
            Some(White)
        } else {
            Some(Black)
        }
    }

    /// Returns Some(Piece) if the square is not empty, None otherwise
    pub fn get_piece_on(&self, square: Square) -> Option<Piece> {
        self.get_piece_type_on(square)
            .map(|piece_type| Piece(piece_type, self.get_piece_color_on(square).unwrap()))
    }

    /// Returns true if specified move is legal for current position
    pub fn is_legal_move(&self, chess_move: BoardMove) -> bool {
        match chess_move {
            BoardMove::MovePiece(m) => {
                let source = m.get_source_square();
                let destination = m.get_destination_square();

                // Check source square
                if (self.get_piece_type_mask(m.get_piece_type())
                    & self.get_color_mask(self.side_to_move)
                    & BitBoard::from_square(source))
                .count_ones()
                    != 1
                {
                    return false;
                }

                // Check destination square availability
                let is_blocked_path = || {
                    let between = BETWEEN.get(source, destination);
                    (between.unwrap() & self.get_combined_mask()).count_ones() > 0
                };
                let destination_mask = match m.get_piece_type() {
                    Pawn => {
                        let en_passant_mask = match self.get_en_passant() {
                            Some(sq) => BitBoard::from_square(sq),
                            None => BLANK,
                        };
                        PAWN.get_moves(source, self.side_to_move)
                            & !self.get_color_mask(self.side_to_move)
                            | PAWN.get_captures(source, self.side_to_move)
                                & (self.get_color_mask(!self.side_to_move) | en_passant_mask)
                    }
                    Knight => KNIGHT.get_moves(source) & !self.get_color_mask(self.side_to_move),
                    Bishop => {
                        if is_blocked_path() {
                            return false;
                        }
                        BISHOP.get_moves(source) & !self.get_color_mask(self.side_to_move)
                    }
                    Rook => {
                        if is_blocked_path() {
                            return false;
                        }
                        ROOK.get_moves(source) & !self.get_color_mask(self.side_to_move)
                    }
                    Queen => {
                        if is_blocked_path() {
                            return false;
                        }
                        QUEEN.get_moves(source) & !self.get_color_mask(self.side_to_move)
                    }
                    King => KING.get_moves(source) & !self.get_color_mask(self.side_to_move),
                };

                if (destination_mask & BitBoard::from_square(destination)).count_ones() != 1 {
                    return false;
                }

                // Check promotions
                if (m.get_promotion().is_some())
                    & (m.get_piece_type() != Pawn)
                    & (destination.get_rank() != self.side_to_move.get_back_rank())
                {
                    return false;
                }

                // Checks
                if self.get_check_mask_after_piece_move(m).count_ones() != 0 {
                    return false;
                }
            }
            BoardMove::CastleKingSide => {
                let is_check = self.get_check_mask().count_ones() > 0;
                if is_check | !self.get_castle_rights(self.side_to_move).has_kingside() {
                    return false;
                }
                let (square_king_side_1, square_king_side_2) = match self.side_to_move {
                    White => (squares::F1, squares::G1),
                    Black => (squares::F8, squares::G8),
                };
                let is_king_side_under_attack = self.is_under_attack(square_king_side_1)
                    | self.is_under_attack(square_king_side_2);
                let is_empty_king_side = ((BitBoard::from_square(square_king_side_1)
                    | BitBoard::from_square(square_king_side_2))
                    & self.get_combined_mask())
                .count_ones()
                    == 0;
                if is_king_side_under_attack | !is_empty_king_side {
                    return false;
                }
            }
            BoardMove::CastleQueenSide => {
                let is_check = self.get_check_mask().count_ones() > 0;
                if is_check | !self.get_castle_rights(self.side_to_move).has_queenside() {
                    return false;
                }

                let (square_queen_side_1, square_queen_side_2) = match self.side_to_move {
                    White => (squares::D1, squares::C1),
                    Black => (squares::D8, squares::C8),
                };
                let is_queen_side_under_attack = self.is_under_attack(square_queen_side_1)
                    | self.is_under_attack(square_queen_side_2);
                let is_empty_queen_side = ((BitBoard::from_square(square_queen_side_1)
                    | BitBoard::from_square(square_queen_side_2))
                    & self.get_combined_mask())
                .count_ones()
                    == 0;
                if is_queen_side_under_attack | !is_empty_queen_side {
                    return false;
                }
            }
        }

        true
    }

    /// Returns true if current side has at least one legal move
    pub fn is_terminal(&self) -> bool { self.is_terminal_position }

    /// Returns a HashSet of all legal moves for current board
    pub fn get_legal_moves(&self) -> LegalMoves {
        let mut moves = LegalMoves::new();
        let color_mask = self.get_color_mask(self.side_to_move);
        let en_passant_mask = match self.get_en_passant() {
            Some(sq) => BitBoard::from_square(sq),
            None => BLANK,
        };
        let promotion_rank = match self.side_to_move {
            White => Rank::Eighth,
            Black => Rank::First,
        };

        let truncate_rays = |mut full_moves_mask: BitBoard, square: Square| {
            let mut legals = BLANK;
            RAYS.get(square).into_iter().for_each(|ray| {
                let mut ray_mask = ray & full_moves_mask;
                (ray_mask & self.combined_mask).for_each(|s| {
                    let between = BETWEEN.get(square, s).unwrap();
                    ray_mask &= between | BitBoard::from_square(s);
                });
                legals |= ray_mask;
            });
            full_moves_mask = legals & !color_mask;
            full_moves_mask
        };

        for i in 0..PIECE_TYPES_NUMBER {
            let piece_type = PieceType::from_index(i).unwrap();
            let free_pieces_mask = color_mask & self.get_piece_type_mask(piece_type);
            for square in free_pieces_mask {
                let full = match piece_type {
                    Pawn => {
                        (PAWN.get_moves(square, self.side_to_move) & !self.combined_mask)
                            | (PAWN.get_captures(square, self.side_to_move)
                                & (self.get_color_mask(!self.side_to_move) | en_passant_mask))
                    }
                    Knight => KNIGHT.get_moves(square) & !color_mask,
                    King => KING.get_moves(square) & !color_mask,
                    Bishop => truncate_rays(BISHOP.get_moves(square), square),
                    Rook => truncate_rays(ROOK.get_moves(square), square),
                    Queen => truncate_rays(QUEEN.get_moves(square), square),
                };

                for m in full
                    .map(|s| PieceMove::new(piece_type, square, s, None).unwrap())
                    .filter(|pm| {
                        self.clone()
                            .move_piece(*pm)
                            .update_pins_and_checks()
                            .get_check_mask()
                            .count_ones()
                            == 0
                    })
                {
                    if (m.get_piece_type() == Pawn)
                        & (m.get_destination_square().get_rank() == promotion_rank)
                    {
                        // Generate promotion moves
                        let (s, d) = (m.get_source_square(), m.get_destination_square());
                        moves.insert(mv!(Pawn, s, d, Knight));
                        moves.insert(mv!(Pawn, s, d, Bishop));
                        moves.insert(mv!(Pawn, s, d, Rook));
                        moves.insert(mv!(Pawn, s, d, Queen));
                    } else {
                        moves.insert(BoardMove::MovePiece(m));
                    }
                }
            }
        }

        // Check if castling is legal
        let is_not_check = self.get_check_mask().count_ones() == 0;
        if is_not_check & self.get_castle_rights(self.side_to_move).has_kingside() {
            let (square_king_side_1, square_king_side_2) = match self.side_to_move {
                White => (squares::F1, squares::G1),
                Black => (squares::F8, squares::G8),
            };
            let is_king_side_under_attack =
                self.is_under_attack(square_king_side_1) | self.is_under_attack(square_king_side_2);
            let is_empty_king_side = ((BitBoard::from_square(square_king_side_1)
                | BitBoard::from_square(square_king_side_2))
                & self.get_combined_mask())
            .count_ones()
                == 0;
            if !is_king_side_under_attack & is_empty_king_side {
                moves.insert(castle_king_side!());
            }
        }

        if is_not_check & self.get_castle_rights(self.side_to_move).has_queenside() {
            let (square_queen_side_1, square_queen_side_2) = match self.side_to_move {
                White => (squares::D1, squares::C1),
                Black => (squares::D8, squares::C8),
            };
            let is_queen_side_under_attack = self.is_under_attack(square_queen_side_1)
                | self.is_under_attack(square_queen_side_2);
            let is_empty_queen_side = ((BitBoard::from_square(square_queen_side_1)
                | BitBoard::from_square(square_queen_side_2))
                & self.get_combined_mask())
            .count_ones()
                == 0;
            if !is_queen_side_under_attack & is_empty_queen_side {
                moves.insert(castle_queen_side!());
            }
        }

        moves
    }

    /// Returns the hash of the position. Is used to detect the repetition draw
    pub fn get_hash(&self) -> PositionHashValueType { self.hash }

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
        } else if self.moves_since_capture_or_pawn_move >= 100 {
            BoardStatus::FiftyMovesDrawDeclared
        } else {
            BoardStatus::Ongoing
        }
    }

    /// Returns true if neither white and black can not checkmate each other
    pub fn is_theoretical_draw_on_board(&self) -> bool {
        let white_pieces_number = self.get_color_mask(White).count_ones();
        let black_pieces_number = self.get_color_mask(Black).count_ones();

        if (white_pieces_number > 2) | (black_pieces_number > 2) {
            return false;
        }

        let bishops_and_knights =
            self.get_piece_type_mask(Knight) | self.get_piece_type_mask(Bishop);

        let white_can_not_checkmate = match white_pieces_number {
            1 => true,
            2 => self.get_color_mask(White) & bishops_and_knights != BLANK,
            _ => unreachable!(),
        };
        let black_can_not_checkmate = match black_pieces_number {
            1 => true,
            2 => self.get_color_mask(Black) & bishops_and_knights != BLANK,
            _ => unreachable!(),
        };

        white_can_not_checkmate & black_can_not_checkmate
    }

    /// This method is needed to represent the chess move without any ambiguity in PGN-like strings
    pub fn get_move_ambiguity_type(
        &self,
        piece_move: PieceMove,
    ) -> Result<DisplayAmbiguityType, Error> {
        use DisplayAmbiguityType::*;

        if !self.is_legal_move(BoardMove::MovePiece(piece_move)) {
            return Err(Error::IllegalMoveDetected);
        }

        let piece_type = piece_move.get_piece_type();
        let source_square = piece_move.get_source_square();
        let destination_square = piece_move.get_destination_square();

        if piece_type == Pawn {
            if source_square.get_file() != destination_square.get_file() {
                return Ok(ExtraFile);
            }
        } else if piece_type == King {
            return Ok(Neither);
        } else {
            let pieces_mask =
                self.get_piece_type_mask(piece_type) & self.get_color_mask(self.side_to_move);
            let piece_moves = match piece_type {
                Knight => KNIGHT.get_moves(destination_square),
                Bishop => BISHOP.get_moves(destination_square),
                Rook => ROOK.get_moves(destination_square),
                Queen => QUEEN.get_moves(destination_square),
                _ => BLANK,
            };

            let between_filter = |x: &Square| match piece_type {
                Knight => true,
                _ => {
                    (BETWEEN
                        .get(*x, piece_move.get_destination_square())
                        .unwrap()
                        & self.combined_mask)
                        .count_ones()
                        == 0
                }
            };

            if (piece_moves & pieces_mask)
                .into_iter()
                .filter(between_filter)
                .count()
                > 1
            {
                if (BitBoard::from_file(source_square.get_file()) & pieces_mask).count_ones() > 1 {
                    return Ok(ExtraRank);
                } else {
                    return Ok(ExtraFile);
                }
            }
        }

        Ok(Neither)
    }

    /// Same as .make_move() but modifies existing board instead of creating new one
    pub fn make_move_mut(&mut self, next_move: BoardMove) -> Result<&mut Self, Error> {
        if !self.is_legal_move(next_move) {
            return Err(Error::IllegalMoveDetected);
        }

        match next_move {
            BoardMove::MovePiece(m) => {
                self.move_piece(m).clear_square_if_en_passant_capture(m);
            }
            BoardMove::CastleKingSide => {
                let king_rank = match self.side_to_move {
                    White => Rank::First,
                    Black => Rank::Eighth,
                };
                self.move_piece(
                    PieceMove::new(
                        King,
                        Square::from_rank_file(king_rank, File::E),
                        Square::from_rank_file(king_rank, File::G),
                        None,
                    )
                    .unwrap(),
                );
                self.move_piece(
                    PieceMove::new(
                        Rook,
                        Square::from_rank_file(king_rank, File::H),
                        Square::from_rank_file(king_rank, File::F),
                        None,
                    )
                    .unwrap(),
                );
            }
            BoardMove::CastleQueenSide => {
                let king_rank = match self.side_to_move {
                    White => Rank::First,
                    Black => Rank::Eighth,
                };
                self.move_piece(
                    PieceMove::new(
                        PieceType::King,
                        Square::from_rank_file(king_rank, File::E),
                        Square::from_rank_file(king_rank, File::C),
                        None,
                    )
                    .unwrap(),
                );
                self.move_piece(
                    PieceMove::new(
                        Rook,
                        Square::from_rank_file(king_rank, File::A),
                        Square::from_rank_file(king_rank, File::D),
                        None,
                    )
                    .unwrap(),
                );
            }
        }

        let new_side_to_move = !self.side_to_move;
        self.update_move_number()
            .update_moves_since_capture(next_move)
            .update_castling_rights(next_move)
            .set_side_to_move(new_side_to_move)
            .update_en_passant(next_move)
            .update_pins_and_checks()
            .update_terminal_status();

        Ok(self)
    }

    /// The method which allows to make moves on the board. Returns a new board instance
    /// if the move is legal
    ///
    /// The simplest way to generate moves is by picking one from a set of available moves:
    /// ``board.get_legal_moves()`` or by simply creating a new move via macros: ``mv!()``,
    /// ``castle_king_side!()`` and ``castle_queen_side!()``
    ///
    /// ```
    /// use libchess::PieceType::*;
    /// use libchess::{castle_king_side, castle_queen_side, mv};
    /// use libchess::{squares::*, BoardMove, ChessBoard, PieceMove};
    ///
    /// let board = ChessBoard::default();
    /// let next_board = board.make_move(mv!(Pawn, E2, E4)).unwrap();
    /// println!("{}", next_board);
    /// ```
    pub fn make_move(&self, next_move: BoardMove) -> Result<Self, Error> {
        let mut next_board = *self;
        next_board.make_move_mut(next_move)?;
        Ok(next_board)
    }

    fn get_check_mask_after_piece_move(self, m: PieceMove) -> BitBoard {
        self.clone()
            .move_piece(m)
            .update_pins_and_checks()
            .get_check_mask()
    }

    fn move_piece(&mut self, piece_move: PieceMove) -> &mut Self {
        let color = self
            .get_piece_color_on(piece_move.get_source_square())
            .unwrap();
        self.clear_square(piece_move.get_source_square())
            .clear_square(piece_move.get_destination_square())
            .put_piece(
                match piece_move.get_promotion() {
                    Some(new_piece_type) => Piece(new_piece_type, color),
                    None => Piece(piece_move.get_piece_type(), color),
                },
                piece_move.get_destination_square(),
            )
    }

    fn clear_square_if_en_passant_capture(&mut self, piece_move: PieceMove) -> &mut Self {
        if let Some(sq) = self.en_passant {
            if (piece_move.get_piece_type() == Pawn) & (piece_move.get_destination_square() == sq) {
                self.clear_square(match self.side_to_move {
                    White => piece_move.get_destination_square().down().unwrap(),
                    Black => piece_move.get_destination_square().up().unwrap(),
                });
            }
        }
        self
    }

    fn set_move_number(&mut self, value: usize) -> &mut Self {
        self.move_number = value;
        self
    }

    fn set_moves_since_capture_or_pawn_move(&mut self, value: usize) -> &mut Self {
        self.moves_since_capture_or_pawn_move = value;
        self
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
        if let Some(piece_type) = self.get_piece_type_on(square) {
            let color = self.get_piece_color_on(square).unwrap();
            let mask = !BitBoard::from_square(square);

            self.combined_mask &= mask;
            self.pieces_mask[piece_type.to_index()] &= mask;
            self.colors_mask[color.to_index()] &= mask;

            self.hash ^= ZOBRIST.get_piece_square_value(Piece(piece_type, color), square);
        }
        self
    }

    fn update_pins_and_checks(&mut self) -> &mut Self {
        let king_square = self.get_king_square(self.side_to_move);
        (self.pinned, self.checks) = self.get_pins_and_checks(king_square);
        self
    }

    fn update_en_passant(&mut self, last_move: BoardMove) -> &mut Self {
        match last_move {
            BoardMove::MovePiece(m) => {
                let src_rank_index = m.get_source_square().get_rank().to_index();
                let dest_rank_index = m.get_destination_square().get_rank().to_index();
                if (m.get_piece_type() == Pawn) & (src_rank_index.abs_diff(dest_rank_index) == 2) {
                    let en_passant_square = Square::from_rank_file(
                        Rank::from_index((src_rank_index + dest_rank_index) / 2).unwrap(),
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
        if !self.get_castle_rights(self.side_to_move).has_any() {
            // check to avoid following code of updating the rights after king loses them
            return self;
        }

        self.set_castling_rights(
            self.side_to_move,
            self.get_castle_rights(self.side_to_move)
                - match last_move {
                    BoardMove::MovePiece(m) => match m.get_piece_type() {
                        Rook => match m.get_source_square().get_file() {
                            File::H => CastlingRights::KingSide,
                            File::A => CastlingRights::QueenSide,
                            _ => CastlingRights::Neither,
                        },
                        King => CastlingRights::BothSides,
                        _ => CastlingRights::Neither,
                    },
                    _ => CastlingRights::BothSides,
                },
        );
        self
    }

    fn update_move_number(&mut self) -> &mut Self {
        if self.side_to_move == Black {
            self.move_number += 1;
        }
        self
    }

    fn update_moves_since_capture(&mut self, last_move: BoardMove) -> &mut Self {
        match last_move {
            BoardMove::MovePiece(m) => {
                if (m.get_piece_type() == Pawn) | m.is_capture_on_board(*self) {
                    self.moves_since_capture_or_pawn_move = 0;
                } else {
                    self.moves_since_capture_or_pawn_move += 1;
                }
            }
            _ => {
                self.moves_since_capture_or_pawn_move = 0;
            }
        }
        self
    }

    fn update_terminal_status(&mut self) -> &mut Self {
        // To define whether the position is terminal one, we should understand that current side
        // does not have legal moves. The simplest way could do this is just by calling
        // board.get_legal_moves().len(). But we could avoid iterating over all available
        // moves for most of the cases and find only the first legal move.
        // Moreover, we do not need to process castling and promotions because for mate and
        // stalemate it is unnecessary
        let color_mask = self.get_color_mask(self.side_to_move);
        let en_passant_mask = match self.get_en_passant() {
            Some(sq) => BitBoard::from_square(sq),
            None => BLANK,
        };

        let truncate_to_first_block = |mut full_moves_mask: BitBoard, square: Square| {
            let mut legals = BLANK;
            RAYS.get(square).into_iter().for_each(|ray| {
                let mut ray_mask = ray & full_moves_mask;
                (ray_mask & self.combined_mask).into_iter().for_each(|s| {
                    let between = BETWEEN.get(square, s).unwrap();
                    ray_mask &= between | BitBoard::from_square(s);
                });
                legals |= ray_mask;
            });
            full_moves_mask = legals & !color_mask;
            full_moves_mask
        };

        for i in 0..PIECE_TYPES_NUMBER {
            let piece_type = PieceType::from_index(i).unwrap();
            let free_pieces_mask =
                color_mask & self.get_piece_type_mask(piece_type) & !self.get_pin_mask();

            for square in free_pieces_mask {
                let full = match piece_type {
                    Pawn => {
                        (PAWN.get_moves(square, self.side_to_move) & !self.combined_mask)
                            | (PAWN.get_captures(square, self.side_to_move)
                                & (self.get_color_mask(!self.side_to_move) | en_passant_mask))
                    }
                    Knight => KNIGHT.get_moves(square) & !color_mask,
                    King => KING.get_moves(square) & !color_mask,
                    Bishop => truncate_to_first_block(BISHOP.get_moves(square), square),
                    Rook => truncate_to_first_block(ROOK.get_moves(square), square),
                    Queen => truncate_to_first_block(QUEEN.get_moves(square), square),
                };

                if full
                    .into_iter()
                    .map(|s| {
                        self.clone()
                            .move_piece(PieceMove::new(piece_type, square, s, None).unwrap())
                            .update_pins_and_checks()
                            .get_check_mask()
                            .count_ones()
                    })
                    .any(|x| x == 0)
                {
                    self.is_terminal_position = false;
                    return self;
                }
            }
        }

        self.is_terminal_position = true;
        self
    }

    fn get_pins_and_checks(&self, square: Square) -> (BitBoard, BitBoard) {
        let opposite_color = !self.side_to_move;
        let bishops_and_queens = self.get_piece_type_mask(Bishop) | self.get_piece_type_mask(Queen);
        let rooks_and_queens = self.get_piece_type_mask(Rook) | self.get_piece_type_mask(Queen);

        let (bishop_mask, rook_mask) = (BISHOP.get_moves(square), ROOK.get_moves(square));
        let pinners = self.get_color_mask(opposite_color)
            & (bishop_mask & bishops_and_queens | rook_mask & rooks_and_queens);

        let (mut pinned, mut checks) = (BLANK, BLANK);
        let mut between;
        for pinner_square in pinners {
            between = self.get_combined_mask() & BETWEEN.get(square, pinner_square).unwrap();
            if between == BLANK {
                checks |= BitBoard::from_square(pinner_square);
            } else if between.count_ones() == 1 {
                pinned |= between;
            }
        }

        checks |= self.get_color_mask(opposite_color)
            & KNIGHT.get_moves(square)
            & self.get_piece_type_mask(Knight);

        checks |= {
            let mut all_pawn_attacks = BLANK;
            (self.get_color_mask(opposite_color) & self.get_piece_type_mask(Pawn))
                .for_each(|sq| all_pawn_attacks |= PAWN.get_captures(sq, opposite_color));
            all_pawn_attacks & BitBoard::from_square(square)
        };

        checks |= self.get_color_mask(opposite_color)
            & KING.get_moves(square)
            & self.get_piece_type_mask(King);

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
    use crate::{squares::*, BoardMove, PieceMove, Square};

    pub fn noindent(text: &str) -> String { text.replace("\n", "").replace(" ", "") }

    #[test]
    fn create_from_string() {
        assert_eq!(
            format!("{}", BoardBuilder::from(ChessBoard::default())),
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
            noindent(
                format!("{}", board)
                    .replace("\u{1b}[47;30m", "")
                    .replace("\u{1b}[47m", "")
                    .replace("\u{1b}[0m", "").as_str()
            ),
            noindent(board_str)
        );

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
            noindent(
                format!("{}", board.render_flipped())
                    .replace("\u{1b}[47;30m", "")
                    .replace("\u{1b}[47m", "")
                    .replace("\u{1b}[0m", "").as_str()
            ),
            noindent(board_str)
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
        let result = 0xffff00000000ffffu64;
        assert_eq!(board.get_combined_mask().bits(), result);

        let result = 0x000000000000ffffu64;
        assert_eq!(board.get_color_mask(Color::White).bits(), result);

        let result = 0xffff000000000000u64;
        assert_eq!(board.get_color_mask(Color::Black).bits(), result);
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
        let mut board = ChessBoard::from_str("1r5k/P7/7K/8/8/8/8/8 w - - 0 1").unwrap();
        for one in board.get_legal_moves() {
            println!("{}", one);
        }
        assert_eq!(board.get_legal_moves().len(), 11);

        board = board
            .make_move(BoardMove::from_str("a7b8=Q").unwrap())
            .unwrap();
        assert_eq!(board.as_fen(), "1Q5k/8/7K/8/8/8/8/8 b - - 0 1");
    }

    #[test]
    fn en_passant() {
        let position =
            ChessBoard::from_str("rnbqkbnr/ppppppp1/8/4P2p/8/8/PPPP1PPP/PNBQKBNR b - - 0 1")
                .unwrap();

        let next_position = position.make_move(mv![Pawn, D7, D5]).unwrap();
        assert!(next_position.get_legal_moves().contains(&mv![Pawn, E5, D6]));

        let position =
            ChessBoard::from_str("4rk2/1p4pp/1pp2q2/r2pb3/3NpP1P/P3P1PR/1PPRQ3/2K5 b - f3 0 27")
                .unwrap();
        assert!(position.get_legal_moves().contains(&mv![Pawn, E4, F3]));
        let next_position = position.make_move(mv![Pawn, E4, F3]).unwrap();
        assert_eq!(
            next_position.as_fen(),
            "4rk2/1p4pp/1pp2q2/r2pb3/3N3P/P3PpPR/1PPRQ3/2K5 w - - 0 28"
        );
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
