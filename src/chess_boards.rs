//! Chess Board module
//!
//! This module defines the representation of position on the board (including Zobrist hash
//! calculation). Implements the logics of moving pieces and inferring the board status

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
use crate::{CastlingRights::*, Color::*, PieceType::*};
use colored::Colorize;
use std::fmt;
use std::str::FromStr;

pub type LegalMoves = Vec<BoardMove>;

/// Represents the board status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardStatus {
    Ongoing,
    CheckMated(Color),
    TheoreticalDrawDeclared,
    FiftyMovesDrawDeclared,
    Stalemate,
}

/// The Chess board representation
///
/// Represents any available board position. Can be initialized by the FEN-string (most recommended)
/// or directly from a BoardBuilder struct. Checks the sanity of the position, so if the struct is
/// created the position is valid. If the initial position is not the terminal (stalemate or
/// checkmate), you can generate another valid board after calling
/// ``board.make_move(&self, next_move: ChessMove)`` (of course, the move must be legal).
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
/// println!("{}", board.make_move(&mv!(King, F4, G5)).unwrap());
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
            .set_castling_rights(White, builder.get_castle_rights(White))
            .set_castling_rights(Black, builder.get_castle_rights(Black))
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
    /// Creates new instance of ChessBoard and fills all parameters without any sanity checks
    fn new() -> Self {
        ChessBoard {
            pieces_mask: [BLANK; PIECE_TYPES_NUMBER],
            colors_mask: [BLANK; COLORS_NUMBER],
            combined_mask: BLANK,
            side_to_move: White,
            castle_rights: [BothSides; COLORS_NUMBER],
            en_passant: None,
            pinned: BLANK,
            checks: BLANK,
            is_terminal_position: false,
            moves_since_capture_or_pawn_move: 0,
            move_number: 1,
            hash: 0,
        }
    }

    /// Creates new instance of ChessBoard by setting up all board's properties manually
    ///
    /// # Errors
    /// ``LibChessError::InvalidPositionColorsOverlap`` if there is any square taken by white and
    /// black color simultaneously
    ///
    /// ``LibChessError::InvalidPositionPieceTypeOverlap`` if there is any square taken by 2 or more
    /// different piece types simultaneously
    ///
    /// ``LibChessError::InvalidBoardMultipleOneColorKings`` if there is more than 1 king of each
    /// color
    ///
    /// ``LibChessError::InvalidBoardOpponentIsOnCheck`` if opponent is on check and it is our move
    ///
    /// ``LibChessError::InvalidBoardInconsistentEnPassant`` if there is not any pawn in front of en
    /// passant square
    ///
    /// ``LibChessError::InvalidBoardInconsistentCastlingRights`` if there is any incompatible
    /// conditions of king an rooks positions and castling rights for any of color
    ///
    /// # Examples
    /// ```
    /// use libchess::*;
    /// use libchess::{squares::*, Color::*, PieceType::*};
    /// let board = ChessBoard::setup(
    ///     &[
    ///         (E1, Piece(King, White)),
    ///         (E8, Piece(King, Black)),
    ///         (E2, Piece(Pawn, White)),
    ///     ], // iterable container of pairs Square + Piece
    ///     White,                   // side to move
    ///     CastlingRights::Neither, // white castling rights
    ///     CastlingRights::Neither, // black castling rights
    ///     None,                    // Optional en-passant square
    ///     0,                       // Moves number since last capture or pawn move
    ///     1,                       // Move number
    /// )
    /// .unwrap();
    /// println!("{}", board);
    /// ```
    pub fn setup<'a>(
        pieces: impl IntoIterator<Item = &'a (Square, Piece)>,
        side_to_move: Color,
        white_castle_rights: CastlingRights,
        black_castle_rights: CastlingRights,
        en_passant: Option<Square>,
        moves_since_capture_or_pawn_move: usize,
        move_number: usize,
    ) -> Result<Self, Error> {
        ChessBoard::try_from(BoardBuilder::setup(
            pieces,
            side_to_move,
            white_castle_rights,
            black_castle_rights,
            en_passant,
            moves_since_capture_or_pawn_move,
            move_number,
        ))
    }

    /// Initializes the ChessBoard structure by a FEN-string
    ///
    /// [FEN-string](https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation)
    /// (Forsyth–Edwards Notation) - standard notation for describing a particular board position
    /// of a chess game. This method parses FEN-string and returns ChessBoard that represents
    /// current position.
    /// This method does the same as ``ChessBoard::from_str()``
    ///
    /// # Errors
    /// ``LibChessError::InvalidPositionColorsOverlap`` if there is any square taken by white and
    /// black color simultaneously
    ///
    /// ``LibChessError::InvalidPositionPieceTypeOverlap`` if there is any square taken by 2 or more
    /// different piece types simultaneously
    ///
    /// ``LibChessError::InvalidBoardMultipleOneColorKings`` if there is more than 1 king of each
    /// color
    ///
    /// ``LibChessError::InvalidBoardOpponentIsOnCheck`` if opponent is on check and it is our move
    ///
    /// ``LibChessError::InvalidBoardInconsistentEnPassant`` if there is not any pawn in front of en
    /// passant square
    ///
    /// ``LibChessError::InvalidBoardInconsistentCastlingRights`` if there is any incompatible
    /// conditions of king an rooks positions and castling rights for any of color
    ///
    /// # Examples
    /// ```
    /// use libchess::ChessBoard;
    /// let position = ChessBoard::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    ///     .expect("Invalid FEN-string");
    /// println!("{}", position);
    /// ```
    pub fn from_fen(fen: &str) -> Result<Self, Error> { Self::from_str(fen) }

    /// Validates the position on the board
    fn validate(&self) -> Option<Error> {
        use squares::*;

        // make sure that is no color overlapping
        if !(self.get_color_mask(White) & self.get_color_mask(Black)).is_blank() {
            return Some(Error::InvalidPositionColorsOverlap);
        };

        // check overlapping of piece type masks
        for i in 0..(PIECE_TYPES_NUMBER - 1) {
            for j in (i + 1)..PIECE_TYPES_NUMBER {
                if !(self.get_piece_type_mask(PieceType::from_index(i).unwrap())
                    & self.get_piece_type_mask(PieceType::from_index(j).unwrap()))
                .is_blank()
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
            if (self.get_piece_type_mask(Pawn)
                & self.get_color_mask(!self.side_to_move)
                & BitBoard::from_square(match !self.side_to_move {
                    White => square.up().unwrap(),
                    Black => square.down().unwrap(),
                }))
            .is_blank()
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

    /// Unified (from white's and black's perspective) method for rendering ChessBoard to terminal
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
    ///
    /// # Examples
    /// ```
    /// use libchess::ChessBoard;
    /// println!("{}", ChessBoard::default()); // simplest option
    /// println!("{}", ChessBoard::default().render_straight()); // will print the same
    /// ```
    pub fn render_straight(&self) -> String {
        let footer = "     a  b  c  d  e  f  g  h";
        self.render(RANKS.iter().rev(), FILES.iter(), footer)
    }

    /// Returns ASCII-representation of the flipped board as a String
    ///
    /// # Examples
    /// ```
    /// use libchess::ChessBoard;
    /// println!("{}", ChessBoard::default()); // render from white's perspective by default
    /// println!("{}", ChessBoard::default().render_flipped()); // will print flipped board
    /// ```
    pub fn render_flipped(&self) -> String {
        let footer = "     h  g  f  e  d  c  b  a";
        self.render(RANKS.iter(), FILES.iter().rev(), footer)
    }

    /// Returns a FEN string of current position
    ///
    /// [FEN-string](https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation)
    /// (Forsyth–Edwards Notation) - standard notation for describing a particular board position
    /// of a chess game. This method transforms ChessBoard to a FEN-string
    ///
    /// # Examples
    /// ```
    /// use libchess::ChessBoard;
    /// let initial_position_fen =
    ///     String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    /// assert_eq!(ChessBoard::default().as_fen(), initial_position_fen);
    /// ```
    #[inline]
    pub fn as_fen(&self) -> String { format!("{}", BoardBuilder::from(*self)) }

    /// Returns a Bitboard mask of same-color pieces
    ///
    /// This method is used to locate all pieces of particular color. Typically is used in
    /// combination with ``.get_combined_mask()`` which returns mask of all pieces on the board
    ///
    /// # Examples
    /// ```
    /// use libchess::{ChessBoard, Color::*};
    /// println! {"{}", ChessBoard::default().get_color_mask(White)};
    /// println! {"{}", ChessBoard::default().get_color_mask(Black)};
    /// ```
    #[inline]
    pub fn get_color_mask(&self, color: Color) -> BitBoard { self.colors_mask[color.to_index()] }

    /// Returns a Bitboard mask for all pieces on the board
    ///
    /// # Examples
    /// ```
    /// use libchess::ChessBoard;
    /// println! {"{}", ChessBoard::default().get_combined_mask()};
    /// ```
    #[inline]
    pub fn get_combined_mask(&self) -> BitBoard { self.combined_mask }

    /// Returns a square for king-piece of specified color
    ///
    /// # Examples
    /// ```
    /// use libchess::{ChessBoard, Color::*};
    /// println! {"{}", ChessBoard::default().get_king_square(White)};
    /// println! {"{}", ChessBoard::default().get_king_square(Black)};
    /// ```
    #[inline]
    pub fn get_king_square(&self, color: Color) -> Square {
        (self.get_piece_type_mask(King) & self.get_color_mask(color)).to_square()
    }

    /// Returns a Bitboard mask for all pieces of the same specified type
    ///
    /// # Examples
    /// ```
    /// use libchess::{ChessBoard, PieceType::*};
    /// println! {"{}", ChessBoard::default().get_piece_type_mask(Rook)};
    /// ```
    #[inline]
    pub fn get_piece_type_mask(&self, piece_type: PieceType) -> BitBoard {
        self.pieces_mask[piece_type.to_index()]
    }

    /// Returns a Bitboard mask for all pieces which pins the king with
    /// color defined by ``board.get_side_to_move()``
    ///
    /// # Examples
    /// ```
    /// use libchess::{ChessBoard, PieceType::*};
    /// let board = ChessBoard::from_fen("7k/6r1/8/8/3Q4/4K3/8/6R1 b - - 0 1").unwrap();
    /// println! {"{}", board};
    /// println! {"{}", board.get_pin_mask()};
    /// ```
    #[inline]
    pub fn get_pin_mask(&self) -> BitBoard { self.pinned }

    /// Returns the castling rights (not the availability of castling) for specified color
    ///
    /// The presence of castling rights does not mean that king can castle at
    /// this move (checks, extra pieces on backrank, etc.).
    ///
    /// # Examples
    /// ```
    /// use libchess::{CastlingRights::*, ChessBoard, Color::*};
    /// let board =
    ///     ChessBoard::from_fen("r1bqk1nr/pppp1ppp/2n5/2b1p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 1")
    ///         .unwrap();
    /// assert_eq!(board.get_castle_rights(White), BothSides); /* Rooks or King not moved, so
    /// castling is legal for both sides but available only to the King side */
    /// assert_eq!(board.castling_is_available_on_board(None), KingSide);
    /// ```
    #[inline]
    pub fn get_castle_rights(&self, color: Color) -> CastlingRights {
        self.castle_rights[color.to_index()]
    }

    /// Shows which side has move now
    ///
    /// # Examples
    /// ```
    /// use libchess::{ChessBoard, Color::*};
    /// assert_eq!(ChessBoard::default().get_side_to_move(), White);
    /// ```
    #[inline]
    pub fn get_side_to_move(&self) -> Color { self.side_to_move }

    /// Returns en-passant square if it exists after the last move
    ///
    /// # Examples
    /// ```
    /// use libchess::{mv, squares::*, BoardMove, ChessBoard, PieceMove, PieceType::*};
    /// let board = ChessBoard::default().make_move(&mv!(Pawn, E2, E4)).unwrap();
    /// assert_eq!(board.get_en_passant(), Some(E3));
    /// ```
    #[inline]
    pub fn get_en_passant(&self) -> Option<Square> { self.en_passant }

    /// Returns a move number (increments every time after black makes move)
    #[inline]
    pub fn get_move_number(&self) -> usize { self.move_number }

    /// Returns a number of moves since last capture or pawn move (is used  to determine the game
    /// termination by the 50-move rule)
    #[inline]
    pub fn get_moves_since_capture_or_pawn_move(&self) -> usize {
        self.moves_since_capture_or_pawn_move
    }

    /// Returns a Bitboard mask for all pieces attacking the king with color defined by
    /// ``board.get_side_to_move()``
    #[inline]
    pub fn get_check_mask(&self) -> BitBoard { self.checks }

    /// Checks if specified square is not taken by any piece
    #[inline]
    pub fn is_empty_square(&self, square: Square) -> bool {
        (self.combined_mask & BitBoard::from_square(square)).is_blank()
    }

    /// Returns available sides for castling for this particular color and position
    ///
    /// # Examples
    /// ```
    /// use libchess::{CastlingRights::*, ChessBoard, Color::*};
    /// let board =
    ///     ChessBoard::from_fen("r1bqk1nr/pppp1ppp/2n5/2b1p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 1")
    ///         .unwrap();
    /// assert_eq!(board.get_castle_rights(White), BothSides);
    /// assert_eq!(board.castling_is_available_on_board(None), KingSide); /* despite white has rights to
    /// castle to both sides, for this position allows to castle only to king side */
    /// ```
    pub fn castling_is_available_on_board(&self, check_mask: Option<BitBoard>) -> CastlingRights {
        use squares::*;

        let mut result = Neither;
        let checks = check_mask.unwrap_or(self.get_check_mask());
        if !checks.is_blank() {
            return result;
        }

        // check castling king side
        if self.get_castle_rights(self.side_to_move).has_kingside() {
            let (square_1, square_2) = match self.side_to_move {
                White => (F1, G1),
                Black => (F8, G8),
            };
            let is_king_side_not_attacked =
                !self.is_under_attack(square_1) & !self.is_under_attack(square_2);
            let is_empty_king_side = ((BitBoard::from_square(square_1)
                ^ BitBoard::from_square(square_2))
                & self.get_combined_mask())
            .is_blank();
            if is_king_side_not_attacked & is_empty_king_side {
                result += KingSide;
            }
        }

        // check castling queen side
        if self.get_castle_rights(self.side_to_move).has_queenside() {
            let (square_1, square_2, square_3) = match self.side_to_move {
                White => (D1, C1, B1),
                Black => (D8, C8, B8),
            };
            let is_queen_side_not_attacked =
                !self.is_under_attack(square_1) & !self.is_under_attack(square_2);
            let is_empty_queen_side = ((BitBoard::from_square(square_1)
                ^ BitBoard::from_square(square_2)
                ^ BitBoard::from_square(square_3))
                & self.get_combined_mask())
            .is_blank();
            if is_queen_side_not_attacked & is_empty_queen_side {
                result += QueenSide;
            }
        }

        result
    }

    /// Returns Some(PieceType) object if the square is not empty, None otherwise
    pub fn get_piece_type_on(&self, square: Square) -> Option<PieceType> {
        if self.is_empty_square(square) {
            return None;
        }

        let bitboard = BitBoard::from_square(square);
        let sum = (1..PIECE_TYPES_NUMBER).fold(0, |acc, i| {
            acc + i * !(self.pieces_mask[i] & bitboard).is_blank() as usize
        });
        Some(PieceType::from_index(sum).unwrap())
    }

    /// Returns Some(Color) object if the square is not empty, None otherwise
    pub fn get_piece_color_on(&self, square: Square) -> Option<Color> {
        if self.is_empty_square(square) {
            return None;
        }

        if (self.get_color_mask(White) & BitBoard::from_square(square)).is_blank() {
            return Some(Black);
        }
        Some(White)
    }

    /// Returns Some(Piece) if the square is not empty, None otherwise
    pub fn get_piece_on(&self, square: Square) -> Option<Piece> {
        let piece_type = self.get_piece_type_on(square)?;
        let color = if (self.get_color_mask(White) & BitBoard::from_square(square)).is_blank() {
            Black
        } else {
            White
        };
        Some(Piece(piece_type, color))
    }

    /// Returns true if specified move is legal for current position
    pub fn is_legal_move(&self, chess_move: &BoardMove) -> bool {
        use BoardMove::*;
        if self.is_terminal() {
            return false;
        }

        match chess_move {
            MovePiece(m) => {
                let source = m.get_source_square();
                let destination = m.get_destination_square();

                // Check if defined peace is really stands on square
                if (self.get_piece_type_mask(m.get_piece_type())
                    & self.get_color_mask(self.side_to_move)
                    & BitBoard::from_square(source))
                .is_blank()
                {
                    return false;
                }

                // Check for chosen piece to move to the destination square
                if (self.get_piece_moves_mask(m.get_piece_type(), source)
                    & BitBoard::from_square(destination))
                .is_blank()
                {
                    return false;
                }

                /* Check if promotion option contain any piece. If yes, we need to ensure that this
                is a Pawn move and the pawn is moving to opposite side's back-rank */
                if (m.get_promotion().is_some())
                    & (m.get_piece_type() != Pawn)
                    & (destination.get_rank() != self.side_to_move.get_back_rank())
                {
                    return false;
                }

                /* If current side's King is in check or it is King's move we must analyze, if on
                the next move the check will disappear. In other cases, actually, we simply can
                accept the move */
                let s = m.get_source_square();
                if !self.get_check_mask().is_blank()
                    | (m.get_piece_type() == King)
                    | m.is_en_passant_move(self)
                    | !(BitBoard::from_square(s) & self.pinned).is_blank()
                {
                    return self
                        .get_check_mask_after_piece_move(&chess_move.piece_move().unwrap())
                        .is_blank();
                }
            }
            CastleKingSide => return self.castling_is_available_on_board(None).has_kingside(),
            CastleQueenSide => return self.castling_is_available_on_board(None).has_queenside(),
        }

        true
    }

    /// Returns true if current side has at least one legal move
    #[inline]
    pub fn is_terminal(&self) -> bool { self.is_terminal_position }

    /// Returns a Vec of all legal moves for current board
    pub fn get_legal_moves(&self) -> LegalMoves {
        let mut moves = Vec::with_capacity(218); /* maximum possible number of legal
                                                 moves in a single position (just to avoid memory reallocations) */
        let color_mask = self.get_color_mask(self.side_to_move);
        let check_mask = self.get_check_mask();

        for piece_type in PieceType::iter() {
            for square in color_mask & self.get_piece_type_mask(piece_type) {
                let piece_moves = self
                    .get_piece_moves_mask(piece_type, square)
                    .map(|s| PieceMove::new(piece_type, square, s, None).unwrap())
                    .filter(|pm| {
                        if !check_mask.is_blank()
                            | (piece_type == King)
                            | pm.is_en_passant_move(self)
                            | !(BitBoard::from_square(pm.get_source_square()) & self.pinned)
                                .is_blank()
                        {
                            return self.get_check_mask_after_piece_move(pm).is_blank();
                        }
                        true
                    });

                if piece_type == Pawn {
                    piece_moves.for_each(|m| {
                        let destination = m.get_destination_square();
                        let promotion_rank = self.side_to_move.get_promotion_rank();
                        if destination.get_rank() == promotion_rank {
                            // Generate promotion moves
                            let (s, d) = (m.get_source_square(), destination);
                            moves.extend_from_slice(&[
                                mv!(Pawn, s, d, Knight),
                                mv!(Pawn, s, d, Bishop),
                                mv!(Pawn, s, d, Rook),
                                mv!(Pawn, s, d, Queen),
                            ]);
                        } else {
                            moves.push(BoardMove::MovePiece(m));
                        }
                    })
                } else {
                    moves.extend(piece_moves.map(|m| BoardMove::MovePiece(m)));
                }
            }
        }

        // Check if castling is legal
        moves.extend_from_slice(
            match self.castling_is_available_on_board(Some(check_mask)) {
                QueenSide => &[castle_queen_side!()],
                KingSide => &[castle_king_side!()],
                BothSides => &[castle_king_side!(), castle_queen_side!()],
                Neither => &[],
            },
        );

        moves
    }

    /// Returns the Zobrist-hash of the position. Is used to detect the repetition draw
    #[inline]
    pub fn get_hash(&self) -> PositionHashValueType { self.hash }

    /// Returns position status on the board
    ///
    /// # Examples
    /// ```
    /// use libchess::{BoardStatus::*, ChessBoard, Color::*};
    /// let board = ChessBoard::from_fen("Q4k2/8/5K2/8/8/8/8/8 b - - 0 1").unwrap();
    /// assert_eq!(board.get_status(), CheckMated(Black));
    /// ```
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

    /// Check sufficiency for both sides to checkmate each other. Is used to determine theoretical
    /// draws
    pub fn is_theoretical_draw_on_board(&self) -> bool {
        let white_pieces_number = self.get_color_mask(White).count_ones();
        let black_pieces_number = self.get_color_mask(Black).count_ones();

        if (white_pieces_number > 2) | (black_pieces_number > 2) {
            return false;
        }

        let bishops_and_knights =
            self.get_piece_type_mask(Knight) | self.get_piece_type_mask(Bishop);

        let white_can_not_checkmate = match white_pieces_number {
            1 => true, // Only white king is on the board
            2 => !(self.get_color_mask(White) & bishops_and_knights).is_blank(), /* only white king and white bishop or knight are on the board */
            _ => unreachable!(),
        };
        let black_can_not_checkmate = match black_pieces_number {
            1 => true, // Only black king is on the board
            2 => !(self.get_color_mask(Black) & bishops_and_knights).is_blank(), /* only black king and white black or knight are on the board */
            _ => unreachable!(),
        };

        white_can_not_checkmate & black_can_not_checkmate
    }

    /// Represents chess moves in short mode without ambiguities in PGN-like strings
    pub fn get_move_ambiguity_type(
        &self,
        piece_move: &PieceMove,
    ) -> Result<DisplayAmbiguityType, Error> {
        use DisplayAmbiguityType::*;

        if !self.is_legal_move(&BoardMove::MovePiece(*piece_move)) {
            return Err(Error::IllegalMoveDetected);
        }

        let piece_type = piece_move.get_piece_type();
        let source = piece_move.get_source_square();
        let destination = piece_move.get_destination_square();

        if piece_type == Pawn {
            if source.get_file() != destination.get_file() {
                return Ok(ExtraFile);
            }
        } else if piece_type == King {
            return Ok(Neither);
        } else {
            let piece_moves = match piece_type {
                Knight => KNIGHT.get_moves(destination),
                Bishop => BISHOP.get_moves(destination),
                Rook => ROOK.get_moves(destination),
                Queen => QUEEN.get_moves(destination),
                _ => unreachable!(),
            };

            let between_filter = |x: &Square| match piece_type {
                Knight => true,
                _ => BETWEEN
                    .get(*x, piece_move.get_destination_square())
                    .map_or(BLANK, |x| x & self.combined_mask)
                    .is_blank(),
            };

            let pieces_mask =
                self.get_piece_type_mask(piece_type) & self.get_color_mask(self.side_to_move);
            if (piece_moves & pieces_mask).filter(between_filter).count() > 1 {
                if (BitBoard::from_file(source.get_file()) & pieces_mask).count_ones() > 1 {
                    return Ok(ExtraRank);
                } else {
                    return Ok(ExtraFile);
                }
            }
        }

        Ok(Neither)
    }

    /// The method which allows to make moves on the board. Modifies the board object if the move
    /// is legal
    ///
    /// The simplest way to generate moves is by picking one from a set of available moves:
    /// ``board.get_legal_moves()`` or by simply creating a new move via macros: ``mv!()``,
    /// ``castle_king_side!()`` and ``castle_queen_side!()``
    ///
    /// # Errors
    /// ``LibChessError::IllegalMoveDetected`` if specified move is not legal
    ///
    /// # Examples
    /// ```
    /// use libchess::PieceType::*;
    /// use libchess::{castle_king_side, castle_queen_side, mv};
    /// use libchess::{squares::*, BoardMove, ChessBoard, PieceMove};
    ///
    /// let mut board = ChessBoard::default();
    /// board.make_move(&mv!(Pawn, E2, E4)).unwrap();
    /// println!("{}", board);
    /// ```
    pub fn make_move_mut(&mut self, next_move: &BoardMove) -> Result<&mut Self, Error> {
        if !self.is_legal_move(next_move) {
            return Err(Error::IllegalMoveDetected);
        }
        unsafe { Ok(self.make_move_mut_unchecked(next_move)) }
    }

    /// The unsafe version of ``ChessBoard::make_move_mut`` method. It does not perform the check if
    /// the move is legal or not. It is only useful for performance reasons during the process of
    /// engine search of the best move. Often used in pair with ``ChessBoard::get_legal_moves``
    /// which guaranties that any provided move will be legal one
    ///
    /// # Safety
    ///
    /// Be very careful while using this method because in case of illegal ``next_move`` it
    /// can lead to application panic or to unpredictable changes of the position. If you are not
    /// 100% sure that ``next_move`` will be a legal move - use ``ChessBoard::make_move_mut``
    /// instead
    pub unsafe fn make_move_mut_unchecked(&mut self, next_move: &BoardMove) -> &mut Self {
        use File::*;

        match next_move {
            BoardMove::MovePiece(m) => {
                self.move_piece(m).clear_square_if_en_passant_capture(m);
            }
            BoardMove::CastleKingSide => {
                let back_rank = self.side_to_move.get_back_rank();
                self.move_piece(
                    &PieceMove::new(
                        King,
                        Square::from_rank_file(back_rank, E),
                        Square::from_rank_file(back_rank, G),
                        None,
                    )
                    .unwrap(),
                );
                self.move_piece(
                    &PieceMove::new(
                        Rook,
                        Square::from_rank_file(back_rank, H),
                        Square::from_rank_file(back_rank, F),
                        None,
                    )
                    .unwrap(),
                );
            }
            BoardMove::CastleQueenSide => {
                let back_rank = self.side_to_move.get_back_rank();
                self.move_piece(
                    &PieceMove::new(
                        PieceType::King,
                        Square::from_rank_file(back_rank, E),
                        Square::from_rank_file(back_rank, C),
                        None,
                    )
                    .unwrap(),
                );
                self.move_piece(
                    &PieceMove::new(
                        Rook,
                        Square::from_rank_file(back_rank, A),
                        Square::from_rank_file(back_rank, D),
                        None,
                    )
                    .unwrap(),
                );
            }
        }

        let opposite_side = !self.side_to_move;
        self.update_move_number()
            .update_moves_since_capture(next_move)
            .update_castling_rights(next_move)
            .set_side_to_move(opposite_side)
            .update_en_passant(next_move)
            .update_pins_and_checks()
            .update_terminal_status();

        self
    }

    /// The method which allows to make moves on the board. Returns a new board instance
    /// if the move is legal
    ///
    /// The simplest way to generate moves is by picking one from a set of available moves:
    /// ``board.get_legal_moves()`` or by simply creating a new move via macros: ``mv!()``,
    /// ``castle_king_side!()`` and ``castle_queen_side!()``
    ///
    /// # Errors
    /// ``LibChessError::IllegalMoveDetected`` if specified move is not legal
    ///
    /// # Examples
    /// ```
    /// use libchess::PieceType::*;
    /// use libchess::{castle_king_side, castle_queen_side, mv};
    /// use libchess::{squares::*, BoardMove, ChessBoard, PieceMove};
    ///
    /// let board = ChessBoard::default();
    /// let next_board = board.make_move(&mv!(Pawn, E2, E4)).unwrap();
    /// println!("{}", next_board);
    /// ```
    pub fn make_move(&self, next_move: &BoardMove) -> Result<Self, Error> {
        let mut next_board = *self;
        next_board.make_move_mut(next_move)?;
        Ok(next_board)
    }

    /// The unsafe version of ``ChessBoard::make_move`` method. It does not perform the check if
    /// the move is legal or not. It is only useful for performance reasons during the process of
    /// engine search of the best move. Often used in pair with ``ChessBoard::get_legal_moves``
    /// which guaranties that any provided move will be legal one
    ///
    /// # Safety
    ///
    /// Be very careful while using this method because in case of illegal ``next_move`` it
    /// can lead to application panic or to unpredictable changes of the position. If you are not
    /// 100% sure that ``next_move`` will be a legal move - use ``ChessBoard::make_move`` instead
    pub unsafe fn make_move_unchecked(&self, next_move: &BoardMove) -> Self {
        let mut next_board = *self;
        next_board.make_move_mut_unchecked(next_move);
        next_board
    }

    fn get_piece_moves_mask(&self, piece_type: PieceType, square: Square) -> BitBoard {
        let color_mask = self.get_color_mask(self.side_to_move);

        let truncate_rays = |pt: PieceType, square: Square| {
            let slice = match pt {
                Bishop => 4..8,
                Rook => 0..4,
                Queen => 0..8,
                _ => unreachable!(),
            };

            let mut legals = BLANK;
            slice.for_each(|i| {
                let ray = RAYS.get(square)[i];
                legals ^= match i {
                    0 | 2 | 4 | 5 => (ray & self.combined_mask).last_bit_square(),
                    1 | 3 | 6 | 7 => (ray & self.combined_mask).first_bit_square(),
                    _ => unreachable!(),
                }
                .map_or(ray, |s| {
                    BETWEEN.get(square, s).unwrap() ^ BitBoard::from_square(s)
                });
            });
            legals & !color_mask
        };

        match piece_type {
            Pawn => {
                let ep = self.get_en_passant().map_or(BLANK, BitBoard::from_square);
                let capturing_squares = self.get_color_mask(!self.side_to_move) | ep;
                let single_move = PAWN.get_moves(square, self.side_to_move) & !self.combined_mask;
                let double_move = if single_move.is_blank() {
                    BLANK
                } else {
                    PAWN.get_double_moves(square, self.side_to_move) & !self.combined_mask
                };

                single_move
                    | double_move
                    | (PAWN.get_captures(square, self.side_to_move) & capturing_squares)
            }
            Knight => KNIGHT.get_moves(square) & !color_mask,
            King => KING.get_moves(square) & !color_mask,
            Bishop => truncate_rays(Bishop, square),
            Rook => truncate_rays(Rook, square),
            Queen => truncate_rays(Queen, square),
        }
    }

    fn get_check_mask_after_piece_move(self, m: &PieceMove) -> BitBoard {
        self.clone()
            .move_piece(m)
            .clear_square_if_en_passant_capture(m)
            .update_pins_and_checks()
            .get_check_mask()
    }

    fn move_piece(&mut self, piece_move: &PieceMove) -> &mut Self {
        let source = piece_move.get_source_square();
        let color = self.get_piece_color_on(source).unwrap();
        self.clear_square(source).put_piece(
            piece_move.get_promotion().map_or(
                Piece(piece_move.get_piece_type(), color),
                |new_piece_type| Piece(new_piece_type, color),
            ),
            piece_move.get_destination_square(),
        )
    }

    fn clear_square_if_en_passant_capture(&mut self, piece_move: &PieceMove) -> &mut Self {
        if piece_move.is_en_passant_move(self) {
            self.clear_square(match self.side_to_move {
                White => piece_move.get_destination_square().down().unwrap(),
                Black => piece_move.get_destination_square().up().unwrap(),
            });
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
            self.side_to_move = color;
        }

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
        if let Some(sq) = self.en_passant {
            self.hash ^= ZOBRIST.get_en_passant_value(sq);
        }
        if let Some(sq) = square {
            self.hash ^= ZOBRIST.get_en_passant_value(sq);
        }

        self.en_passant = square;
        self
    }

    fn put_piece(&mut self, piece: Piece, square: Square) -> &mut Self {
        if !self.is_empty_square(square) {
            self.clear_square(square);
        }
        let mask = BitBoard::from_square(square);
        self.combined_mask ^= mask;
        self.pieces_mask[piece.0.to_index()] ^= mask;
        self.colors_mask[piece.1.to_index()] ^= mask;
        self.hash ^= ZOBRIST.get_piece_square_value(piece, square);
        self
    }

    fn clear_square(&mut self, square: Square) -> &mut Self {
        if let Some(piece) = self.get_piece_on(square) {
            let mask = !BitBoard::from_square(square);
            self.combined_mask &= mask;
            self.pieces_mask[piece.0.to_index()] &= mask;
            self.colors_mask[piece.1.to_index()] &= mask;
            self.hash ^= ZOBRIST.get_piece_square_value(piece, square);
        }
        self
    }

    fn update_pins_and_checks(&mut self) -> &mut Self {
        (self.pinned, self.checks) =
            self.get_pins_and_checks(self.get_king_square(self.side_to_move));
        self
    }

    fn update_en_passant(&mut self, last_move: &BoardMove) -> &mut Self {
        match last_move {
            BoardMove::MovePiece(m) => {
                let src_rank_index = m.get_source_square().get_rank().to_index();
                let dest_rank_index = m.get_destination_square().get_rank().to_index();
                if (m.get_piece_type() == Pawn) & (src_rank_index.abs_diff(dest_rank_index) == 2) {
                    let en_passant_square = Square::from_rank_file(
                        Rank::from_index((src_rank_index + dest_rank_index) / 2).unwrap(),
                        m.get_destination_square().get_file(),
                    );
                    self.set_en_passant(Some(en_passant_square))
                } else {
                    self.set_en_passant(None)
                }
            }
            _ => self.set_en_passant(None),
        };
        self
    }

    fn update_castling_rights(&mut self, last_move: &BoardMove) -> &mut Self {
        use File::*;
        let opposite = !self.side_to_move;

        if (self.get_castle_rights(opposite) != Neither) & last_move.piece_move().is_ok() {
            let destination = last_move.piece_move().unwrap().get_destination_square();
            let opposite_back_rank = opposite.get_back_rank();
            self.set_castling_rights(
                opposite,
                self.get_castle_rights(opposite)
                    - if destination == Square::from_rank_file(opposite_back_rank, H) {
                        KingSide
                    } else if destination == Square::from_rank_file(opposite_back_rank, A) {
                        QueenSide
                    } else {
                        Neither
                    },
            );
        }

        if self.get_castle_rights(self.side_to_move) != Neither {
            self.set_castling_rights(
                self.side_to_move,
                self.get_castle_rights(self.side_to_move)
                    - match last_move {
                        BoardMove::MovePiece(m) => match m.get_piece_type() {
                            Rook => match m.get_source_square().get_file() {
                                File::H => KingSide,
                                File::A => QueenSide,
                                _ => Neither,
                            },
                            King => BothSides,
                            _ => Neither,
                        },
                        _ => BothSides,
                    },
            );
        }

        self
    }

    fn update_move_number(&mut self) -> &mut Self {
        if self.side_to_move == Black {
            self.move_number += 1;
        }
        self
    }

    fn update_moves_since_capture(&mut self, last_move: &BoardMove) -> &mut Self {
        match last_move {
            BoardMove::MovePiece(m) => {
                if (m.get_piece_type() == Pawn) | m.is_capture_on_board(self) {
                    self.moves_since_capture_or_pawn_move = 0;
                } else {
                    self.moves_since_capture_or_pawn_move += 1;
                }
            }
            _ => {
                self.moves_since_capture_or_pawn_move += 1;
            }
        }
        self
    }

    fn update_terminal_status(&mut self) -> &mut Self {
        // To define whether the position is terminal one, we should understand that current side
        // does not have legal moves. The simplest way to do this is just by calling
        // board.get_legal_moves().len(). But we could avoid iterating over all available
        // moves for most of the cases and find only the first legal move.
        // Moreover, we do not need to process castling and promotions because for checkmate and
        // stalemate it is unnecessary
        let color_mask = self.get_color_mask(self.side_to_move);
        for piece_type in PieceType::iter() {
            for square in color_mask & self.get_piece_type_mask(piece_type) {
                if self
                    .get_piece_moves_mask(piece_type, square)
                    .into_iter()
                    .map(|s| {
                        self.get_check_mask_after_piece_move(
                            &PieceMove::new(piece_type, square, s, None).unwrap(),
                        )
                    })
                    .any(|x| x.is_blank())
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
        let opposite = !self.side_to_move;
        let bishops_and_queens = self.get_piece_type_mask(Bishop) | self.get_piece_type_mask(Queen);
        let rooks_and_queens = self.get_piece_type_mask(Rook) | self.get_piece_type_mask(Queen);

        let (diagonal_moves, rect_moves) = (BISHOP.get_moves(square), ROOK.get_moves(square));
        let attackers = self.get_color_mask(opposite)
            & (diagonal_moves & bishops_and_queens | rect_moves & rooks_and_queens);

        let (mut pinned, mut checks) = (BLANK, BLANK);
        for attacker in attackers {
            let between = self.get_combined_mask() & BETWEEN.get(square, attacker).unwrap();
            match between.count_ones() {
                0 => checks |= BitBoard::from_square(attacker),
                1 => pinned |= between,
                _ => {}
            }
        }
        pinned &= self.get_color_mask(self.side_to_move);

        checks |= self.get_color_mask(opposite)
            & (KNIGHT.get_moves(square) & self.get_piece_type_mask(Knight)
                | KING.get_moves(square) & self.get_piece_type_mask(King));

        checks |= {
            let mut pawns_attacks = BLANK;
            if let Ok(rank) = match self.side_to_move {
                White => square.up(),
                Black => square.down(),
            }
            .map(|x| x.get_rank())
            {
                let opposite_pawns = self.get_color_mask(opposite) & self.get_piece_type_mask(Pawn);

                if let Ok(file) = square.left().map(|x| x.get_file()) {
                    let pawn_square = Square::from_rank_file(rank, file);
                    pawns_attacks |= opposite_pawns & BitBoard::from_square(pawn_square);
                }

                if let Ok(file) = square.right().map(|x| x.get_file()) {
                    let pawn_square = Square::from_rank_file(rank, file);
                    pawns_attacks |= opposite_pawns & BitBoard::from_square(pawn_square);
                }
            }
            pawns_attacks
        };

        (pinned, checks)
    }

    fn is_under_attack(&self, square: Square) -> bool {
        !self.get_pins_and_checks(square).1.is_blank()
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
    fn piece_types_and_colors() {
        let board = ChessBoard::default();
        assert_eq!(board.get_piece_type_on(A2).unwrap(), Pawn);
        assert_eq!(board.get_piece_type_on(A1).unwrap(), Rook);
        assert_eq!(board.get_piece_type_on(B1).unwrap(), Knight);
        assert_eq!(board.get_piece_type_on(C1).unwrap(), Bishop);
        assert_eq!(board.get_piece_type_on(E1).unwrap(), King);
        assert_eq!(board.get_piece_type_on(D1).unwrap(), Queen);

        assert_eq!(board.get_piece_color_on(A1).unwrap(), White);
        assert_eq!(board.get_piece_color_on(A8).unwrap(), Black);
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
        another_board = another_board.make_move(&mv!(Pawn, E2, E4)).unwrap();
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

        assert_eq!(
            ChessBoard::from_str(
                "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"
            )
            .unwrap()
            .get_legal_moves()
            .len(),
            48
        );

        ChessBoard::from_str(
            "r1bqkbnr/p2p1ppP/1N2B3/1Pp1Q3/8/P1n2N2/2PPPPP1/R1B1K2R w KQkq c6 0 1",
        )
        .unwrap()
        .get_legal_moves()
        .into_iter()
        .for_each(|x| println!("{x}"));

        assert_eq!(
            ChessBoard::from_str(
                "r1bqkbnr/p2p1ppP/1N2B3/1Pp1Q3/8/P1n2N2/2PPPPP1/R1B1K2R w KQkq c6 0 1"
            )
            .unwrap()
            .get_legal_moves()
            .len(),
            13 + 11 + 10 + 9 + 2 + 17
        );

        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
        let board = ChessBoard::from_str(fen).unwrap();
        board
            .get_legal_moves()
            .iter()
            .for_each(|one| assert_eq!(board.is_legal_move(one), true));
    }

    #[test]
    fn promotion() {
        let mut board = ChessBoard::from_str("1r5k/P7/7K/8/8/8/8/8 w - - 0 1").unwrap();
        for one in board.get_legal_moves() {
            println!("{}", one);
        }
        assert_eq!(board.get_legal_moves().len(), 11);

        board = board
            .make_move(&BoardMove::from_str("a7b8=Q").unwrap())
            .unwrap();
        assert_eq!(board.as_fen(), "1Q5k/8/7K/8/8/8/8/8 b - - 0 1");
    }

    #[test]
    fn en_passant() {
        let position =
            ChessBoard::from_str("rnbqkbnr/ppppppp1/8/4P2p/8/8/PPPP1PPP/PNBQKBNR b - - 0 1")
                .unwrap();

        let next_position = position.make_move(&mv![Pawn, D7, D5]).unwrap();
        assert!(next_position.get_legal_moves().contains(&mv![Pawn, E5, D6]));

        let position =
            ChessBoard::from_str("4rk2/1p4pp/1pp2q2/r2pb3/3NpP1P/P3P1PR/1PPRQ3/2K5 b - f3 0 27")
                .unwrap();
        assert!(position.get_legal_moves().contains(&mv![Pawn, E4, F3]));
        let next_position = position.make_move(&mv![Pawn, E4, F3]).unwrap();
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

        let board = ChessBoard::from_str("r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1")
            .unwrap()
            .make_move(&mv!(Rook, A1, A8))
            .unwrap();
        assert!(board.get_castle_rights(White).has_kingside());
        assert!(!board.get_castle_rights(White).has_queenside());
        assert!(board.get_castle_rights(Black).has_kingside());
        assert!(!board.get_castle_rights(Black).has_queenside());
    }

    #[test]
    fn kill_the_king() {
        assert!(ChessBoard::from_str("Q3k3/8/4K3/8/8/8/8/8 w - - 0 1").is_err());
    }

    fn perft_get_branches(position: &ChessBoard) -> Vec<(BoardMove, ChessBoard)> {
        position
            .get_legal_moves()
            .into_iter()
            .map(|m| (m, position.make_move(&m).unwrap()))
            .collect::<Vec<_>>()
    }

    fn perft_calculate_positions(position: ChessBoard, recursion_level: usize) -> Vec<usize> {
        let mut boards = vec![position];
        let mut positions_counter = vec![0; recursion_level];

        for i in 0..recursion_level {
            let mut x = vec![];
            boards.iter().for_each(|b| {
                let t = perft_get_branches(b);
                x.append(&mut t.clone().into_iter().map(|x| x.1).collect());
            });

            boards = x;
            positions_counter[i] = boards.len();
        }
        positions_counter
    }

    #[test]
    fn perft_1() {
        const MOVES_NUMBER: usize = 5; // Can be tuned in range 1..=5 (affects testing time)
        let position = ChessBoard::default();

        perft_calculate_positions(position, MOVES_NUMBER)
            .into_iter()
            .zip([20, 400, 8902, 197281, 4865609].into_iter())
            .for_each(|(a, b)| assert_eq!(a, b));
    }

    #[test]
    fn perft_2() {
        const MOVES_NUMBER: usize = 4; // Can be tuned in range 1..=5 (affects testing time)
        let position = ChessBoard::from_str(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        )
        .unwrap();

        perft_calculate_positions(position, MOVES_NUMBER)
            .into_iter()
            .zip([48, 2039, 97862, 4085603, 193690690].into_iter())
            .for_each(|(a, b)| assert_eq!(a, b));
    }

    #[test]
    fn perft_3() {
        const MOVES_NUMBER: usize = 5; // Can be tuned in range 1..=5 (affects testing time)
        let position = ChessBoard::from_str("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap();

        perft_calculate_positions(position, MOVES_NUMBER)
            .into_iter()
            .zip([14, 191, 2812, 43238, 674624].into_iter())
            .for_each(|(a, b)| assert_eq!(a, b));
    }

    #[test]
    fn perft_4() {
        const MOVES_NUMBER: usize = 4; // Can be tuned in range 1..=5 (affects testing time)
        let position = ChessBoard::from_str(
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        )
        .unwrap();

        perft_calculate_positions(position, MOVES_NUMBER)
            .into_iter()
            .zip([6, 264, 9467, 422333, 15833292].into_iter())
            .for_each(|(a, b)| assert_eq!(a, b));
    }

    #[test]
    fn perft_5() {
        const MOVES_NUMBER: usize = 4; // Can be tuned in range 1..=5 (affects testing time)
        let position =
            ChessBoard::from_str("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8")
                .unwrap();

        perft_calculate_positions(position, MOVES_NUMBER)
            .into_iter()
            .zip([44, 1486, 62379, 2103487, 89941194].into_iter())
            .for_each(|(a, b)| assert_eq!(a, b));
    }

    #[test]
    fn perft_6() {
        const MOVES_NUMBER: usize = 4; // Can be tuned in range 1..=5 (affects testing time)
        let position = ChessBoard::from_str(
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        )
        .unwrap();

        perft_calculate_positions(position, MOVES_NUMBER)
            .into_iter()
            .zip([46, 2079, 89890, 3894594, 164075551].into_iter())
            .for_each(|(a, b)| assert_eq!(a, b));
    }
}
