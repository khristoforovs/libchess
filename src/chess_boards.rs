use crate::bitboards::{BitBoard, BLANK};
use crate::board_builders::BoardBuilder;
use crate::board_files::{File, FILES};
use crate::board_ranks::{Rank, RANKS};
use crate::castling::CastlingRights;
use crate::chess_moves::{ChessMove, PieceMove, PromotionPieceType};
use crate::colors::{Color, COLORS_NUMBER};
use crate::errors::ChessBoardError as Error;
use crate::move_masks::{
    BETWEEN_TABLE as BETWEEN, BISHOP_TABLE as BISHOP, KING_TABLE as KING, KNIGHT_TABLE as KNIGHT,
    PAWN_TABLE as PAWN, QUEEN_TABLE as QUEEN, ROOK_TABLE as ROOK,
};
use crate::pieces::{Piece, PieceType, NUMBER_PIECE_TYPES};
use crate::squares::{Square, SQUARES_NUMBER};
use crate::{castle_king_side, castle_queen_side, mv};
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
    }
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
            .set_black_moved_counter(builder.get_black_moved_counter())
            .set_moves_since_capture(builder.get_moves_since_capture())
            .update_pins_and_checks()
            .update_legal_moves();

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
    fn new() -> Self {
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
        let white_rook_mask =
            self.get_piece_type_mask(PieceType::Rook) & self.get_color_mask(Color::White);
        if self.get_king_square(Color::White) == Square::E1 {
            let validation_mask = match self.get_castle_rights(Color::White) {
                CastlingRights::Neither => BLANK,
                CastlingRights::QueenSide => BitBoard::from_square(Square::A1),
                CastlingRights::KingSide => BitBoard::from_square(Square::H1),
                CastlingRights::BothSides => {
                    BitBoard::from_square(Square::A1) | BitBoard::from_square(Square::H1)
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
        if self.get_king_square(Color::Black) == Square::E8 {
            let validation_mask = match self.get_castle_rights(Color::Black) {
                CastlingRights::Neither => BLANK,
                CastlingRights::QueenSide => BitBoard::from_square(Square::A8),
                CastlingRights::KingSide => BitBoard::from_square(Square::H8),
                CastlingRights::BothSides => {
                    BitBoard::from_square(Square::A8) | BitBoard::from_square(Square::H8)
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
            next_position
                .update_black_moved_counter()
                .update_moves_since_capture(next_move);
            match next_move {
                ChessMove::MovePiece(m) => {
                    next_position.move_piece(m);
                }
                ChessMove::CastleKingSide => {
                    let king_rank = match self.side_to_move {
                        Color::White => Rank::First,
                        Color::Black => Rank::Eighth,
                    };
                    next_position.move_piece(PieceMove::new(
                        PieceType::King,
                        Square::from_rank_file(king_rank, File::E),
                        Square::from_rank_file(king_rank, File::G),
                        None,
                    ));
                    next_position.move_piece(PieceMove::new(
                        PieceType::Rook,
                        Square::from_rank_file(king_rank, File::H),
                        Square::from_rank_file(king_rank, File::F),
                        None,
                    ));
                }
                ChessMove::CastleQueenSide => {
                    let king_rank = match self.side_to_move {
                        Color::White => Rank::First,
                        Color::Black => Rank::Eighth,
                    };
                    next_position.move_piece(PieceMove::new(
                        PieceType::King,
                        Square::from_rank_file(king_rank, File::E),
                        Square::from_rank_file(king_rank, File::C),
                        None,
                    ));
                    next_position.move_piece(PieceMove::new(
                        PieceType::Rook,
                        Square::from_rank_file(king_rank, File::A),
                        Square::from_rank_file(king_rank, File::D),
                        None,
                    ));
                }
            }
            next_position
                .update_castling_rights(next_move)
                .set_side_to_move(!self.side_to_move)
                .update_en_passant(next_move)
                .update_pins_and_checks()
                .update_legal_moves();
        } else {
            return Err(Error::IllegalMoveDetected);
        }
        Ok(next_position)
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

    fn set_black_moved_counter(&mut self, value: usize) -> &mut Self {
        self.black_moved_counter = value;
        self
    }

    fn set_moves_since_capture(&mut self, value: usize) -> &mut Self {
        self.moves_since_capture_counter = value;
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

    fn put_piece(&mut self, piece: Piece, square: Square) -> &mut Self {
        self.clear_square(square);
        let square_bitboard = BitBoard::from_square(square);
        self.combined_mask |= square_bitboard;
        self.pieces_mask[piece.0.to_index()] |= square_bitboard;
        self.colors_mask[piece.1.to_index()] |= square_bitboard;
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
            }
            None => {}
        }
        self
    }

    fn update_black_moved_counter(&mut self) -> &mut Self {
        if self.side_to_move == Color::Black {
            self.black_moved_counter += 1;
        }
        self
    }

    fn update_moves_since_capture(&mut self, last_move: ChessMove) -> &mut Self {
        match last_move {
            ChessMove::MovePiece(m) => {
                if (m.get_piece_type() == PieceType::Pawn) | m.is_capture_on_board(self) {
                    self.moves_since_capture_counter = 0;
                } else {
                    self.moves_since_capture_counter += 1;
                }
            }
            ChessMove::CastleKingSide | ChessMove::CastleQueenSide => {
                self.moves_since_capture_counter = 0;
            }
        }
        self
    }

    fn update_pins_and_checks(&mut self) -> &mut Self {
        let king_square = self.get_king_square(self.side_to_move);
        (self.pinned, self.checks) = self.get_pins_and_checks(king_square);
        self
    }

    fn update_en_passant(&mut self, last_move: ChessMove) -> &mut Self {
        match last_move {
            ChessMove::MovePiece(m) => {
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
            ChessMove::CastleKingSide | ChessMove::CastleQueenSide => {
                self.set_en_passant(None);
            }
        }
        self
    }

    fn update_castling_rights(&mut self, last_move: ChessMove) -> &mut Self {
        self.set_castling_rights(
            self.side_to_move,
            self.get_castle_rights(self.side_to_move)
                - match last_move {
                    ChessMove::MovePiece(m) => match m.get_piece_type() {
                        PieceType::Rook => match m.get_source_square().get_file() {
                            File::H => CastlingRights::KingSide,
                            File::A => CastlingRights::QueenSide,
                            _ => CastlingRights::Neither,
                        },
                        PieceType::King => CastlingRights::BothSides,
                        _ => CastlingRights::Neither,
                    },
                    ChessMove::CastleKingSide => CastlingRights::BothSides,
                    ChessMove::CastleQueenSide => CastlingRights::BothSides,
                },
        );
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

    fn update_legal_moves(&mut self) -> &mut Self {
        let mut moves = LegalMoves::new();
        let color_mask = self.get_color_mask(self.side_to_move);
        let en_passant_mask = match self.get_en_passant() {
            Some(sq) => BitBoard::from_square(sq),
            None => BLANK,
        };

        for i in 0..NUMBER_PIECE_TYPES {
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
                Color::White => (Square::F1, Square::G1),
                Color::Black => (Square::F8, Square::G8),
            };
            let is_king_side_under_attack =
                self.is_under_attack(square_king_side_1) | self.is_under_attack(square_king_side_2);
            let king_side_between_mask = match self.side_to_move {
                Color::White => BETWEEN.get(Square::E1, Square::H1).unwrap(),
                Color::Black => BETWEEN.get(Square::E8, Square::H8).unwrap(),
            };
            let is_empty_king_side =
                (king_side_between_mask & self.get_combined_mask()).count_ones() == 0;
            if is_not_check & !is_king_side_under_attack & is_empty_king_side {
                moves.insert(castle_king_side!());
            }
        }

        if self.get_castle_rights(self.side_to_move).has_queenside() {
            let (square_queen_side_1, square_queen_side_2) = match self.side_to_move {
                Color::White => (Square::D1, Square::C1),
                Color::Black => (Square::D8, Square::C8),
            };
            let is_queen_side_under_attack = self.is_under_attack(square_queen_side_1)
                | self.is_under_attack(square_queen_side_2);
            let queen_side_between_mask = match self.side_to_move {
                Color::White => BETWEEN.get(Square::E1, Square::A1).unwrap(),
                Color::Black => BETWEEN.get(Square::E8, Square::A8).unwrap(),
            };
            let is_empty_queen_side =
                (queen_side_between_mask & self.get_combined_mask()).count_ones() == 0;
            if is_not_check & !is_queen_side_under_attack & is_empty_queen_side {
                moves.insert(castle_queen_side!());
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

    #[test]
    fn en_passant() {
        let board =
            ChessBoard::from_str("rnbqkbnr/ppppppp1/8/4P2p/8/8/PPPP1PPP/PNBQKBNR b - - 0 1")
                .unwrap();

        let next_board = board
            .make_move(mv![PieceType::Pawn, Square::D7, Square::D5])
            .unwrap();

        assert!(next_board.get_legal_moves().contains(&mv![
            PieceType::Pawn,
            Square::E5,
            Square::D6
        ]));
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
}
