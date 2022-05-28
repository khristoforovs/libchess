use crate::boards::{BitBoard, ChessBoard, Square};
use crate::PieceType;
use std::fmt;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PromotionPieceType {
    Knight,
    Bishop,
    Rook,
    Queen,
}

impl From<PromotionPieceType> for PieceType {
    fn from(piece_type: PromotionPieceType) -> Self {
        match piece_type {
            PromotionPieceType::Knight => PieceType::Knight,
            PromotionPieceType::Bishop => PieceType::Bishop,
            PromotionPieceType::Rook => PieceType::Rook,
            PromotionPieceType::Queen => PieceType::Queen,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DisplayAmbiguityType {
    ExtraFile,
    ExtraSquare,
    Neither,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PieceMove {
    piece_type: PieceType,
    square_from: Square,
    square_to: Square,
    promotion: Option<PieceType>,
}

impl fmt::Display for PieceMove {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let promotion_string = match self.get_promotion() {
            Some(piece_type) => format!("={}", piece_type),
            None => String::new(),
        };
        let piece_type_string = match self.get_piece_type() {
            PieceType::Pawn => String::new(),
            p => format!("{}", p),
        };
        write!(
            f,
            "{}{}{}{}",
            piece_type_string,
            self.get_source_square(),
            self.get_destination_square(),
            promotion_string,
        )
    }
}

impl PieceMove {
    pub fn new(
        piece_type: PieceType,
        square_from: Square,
        square_to: Square,
        promotion: Option<PromotionPieceType>,
    ) -> Self {
        PieceMove {
            piece_type,
            square_from,
            square_to,
            promotion: {
                match promotion {
                    Some(p) => Some(PieceType::from(p)),
                    None => None,
                }
            },
        }
    }

    #[inline]
    pub fn get_piece_type(&self) -> PieceType {
        self.piece_type
    }

    #[inline]
    pub fn get_source_square(&self) -> Square {
        self.square_from
    }

    #[inline]
    pub fn get_destination_square(&self) -> Square {
        self.square_to
    }

    #[inline]
    pub fn get_promotion(&self) -> Option<PieceType> {
        self.promotion
    }

    pub fn is_capture_on_board(&self, board: &ChessBoard) -> bool {
        if (BitBoard::from_square(self.get_destination_square())
            & board.get_color_mask(!board.get_side_to_move()))
        .count_ones()
            > 0
        {
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoardMoveOption {
    MovePiece(PieceMove),
    CastleKingSide,
    CastleQueenSide,
}

#[derive(Debug, Clone, Copy, Eq)]
pub struct BoardMove {
    board_move_option: BoardMoveOption,
    is_capture: Option<bool>,
    is_check: Option<bool>,
    is_checkmate: Option<bool>,
    display_ambiguity_type: DisplayAmbiguityType,
}

impl fmt::Display for BoardMove {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let is_associated = self.is_check().is_some();

        let check_string = if !is_associated {
            String::new()
        } else if self.is_checkmate().unwrap() {
            String::from("#")
        } else if self.is_check().unwrap() {
            String::from("+")
        } else {
            String::new()
        };

        match self.board_move_option {
            BoardMoveOption::MovePiece(m) => {
                let piece_type_string = match m.get_piece_type() {
                    PieceType::Pawn => String::new(),
                    p => format!("{}", p),
                };
                let ambiguity_resolve_string = match self.display_ambiguity_type {
                    DisplayAmbiguityType::ExtraFile => {
                        format!("{}", m.get_source_square().get_file())
                    }
                    DisplayAmbiguityType::ExtraSquare => format!("{}", m.get_source_square()),
                    DisplayAmbiguityType::Neither => String::new(),
                };
                let capture_string = if !is_associated {
                    String::new()
                } else if self.is_capture().unwrap() {
                    String::from("x")
                } else {
                    String::new()
                };
                let promotion_string = match m.get_promotion() {
                    Some(piece_type) => format!("={}", piece_type),
                    None => String::new(),
                };

                write!(
                    f,
                    "{}{}{}{}{}{}",
                    piece_type_string,
                    ambiguity_resolve_string,
                    capture_string,
                    m.get_destination_square(),
                    promotion_string,
                    check_string,
                )
            }
            BoardMoveOption::CastleKingSide => write!(f, "O-O{}", check_string),
            BoardMoveOption::CastleQueenSide => write!(f, "O-O-O{}", check_string),
        }
    }
}

impl PartialEq for BoardMove {
    fn eq(&self, other: &Self) -> bool {
        self.board_move_option == other.board_move_option
    }
}

impl Hash for BoardMove {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.board_move_option.hash(state);
    }
}

impl BoardMove {
    pub fn new(board_move_option: BoardMoveOption) -> Self {
        Self {
            board_move_option,
            is_capture: None,
            is_check: None,
            is_checkmate: None,
            display_ambiguity_type: DisplayAmbiguityType::ExtraSquare,
        }
    }

    pub fn piece_move(&self) -> Option<PieceMove> {
        match self.board_move_option {
            BoardMoveOption::MovePiece(m) => Some(m),
            BoardMoveOption::CastleKingSide => None,
            BoardMoveOption::CastleQueenSide => None,
        }
    }

    #[inline]
    pub fn get_move_option(&self) -> BoardMoveOption {
        self.board_move_option
    }

    #[inline]
    pub fn is_capture(&self) -> Option<bool> {
        self.is_capture
    }

    #[inline]
    pub fn is_check(&self) -> Option<bool> {
        self.is_check
    }

    #[inline]
    pub fn is_checkmate(&self) -> Option<bool> {
        self.is_checkmate
    }

    pub fn associate(
        &mut self,
        board_before_move: &ChessBoard,
        board_after_move: &ChessBoard,
    ) -> &mut Self {
        let is_check = board_after_move.get_check_mask().count_ones() > 0;

        self.is_check = Some(is_check);
        self.is_capture = match self.board_move_option {
            BoardMoveOption::MovePiece(m) => Some(m.is_capture_on_board(board_before_move)),
            BoardMoveOption::CastleKingSide => Some(false),
            BoardMoveOption::CastleQueenSide => Some(false),
        };
        self.is_checkmate = Some(board_after_move.is_terminal() & is_check);
        self.display_ambiguity_type = match self.board_move_option {
            BoardMoveOption::MovePiece(m) => match m.get_piece_type() {
                PieceType::King => DisplayAmbiguityType::Neither,
                _ => board_before_move.get_move_ambiguity_type(m).unwrap(),
            },
            BoardMoveOption::CastleKingSide => DisplayAmbiguityType::Neither,
            BoardMoveOption::CastleQueenSide => DisplayAmbiguityType::Neither,
        };

        self
    }
}

#[macro_export]
macro_rules! mv {
    ($piece_type:expr, $square_from:expr, $square_to:expr) => {
        BoardMove::new(BoardMoveOption::MovePiece(PieceMove::new(
            $piece_type,
            $square_from,
            $square_to,
            None,
        )))
    };

    ($piece_type:expr, $square_from:expr, $square_to:expr, $promotion:expr) => {
        BoardMove::new(BoardMoveOption::MovePiece(PieceMove::new(
            $piece_type,
            $square_from,
            $square_to,
            Some($promotion),
        )))
    };
}

#[macro_export]
macro_rules! castle_king_side {
    () => {
        BoardMove::new(BoardMoveOption::CastleKingSide)
    };
}

#[macro_export]
macro_rules! castle_queen_side {
    () => {
        BoardMove::new(BoardMoveOption::CastleQueenSide)
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn move_representation() {
        let board_move = mv!(PieceType::Pawn, Square::E2, Square::E4);
        assert_eq!(format!("{}", board_move), "e2e4");

        let board_move = mv!(
            PieceType::Pawn,
            Square::E7,
            Square::E8,
            PromotionPieceType::Queen
        );
        assert_eq!(format!("{}", board_move), "e7e8=Q");

        let board_move = mv!(PieceType::Queen, Square::A1, Square::A8);
        assert_eq!(format!("{}", board_move), "Qa1a8");

        let board_move = castle_queen_side!();
        assert_eq!(format!("{}", board_move), "O-O-O");
    }

    #[test]
    fn capture() {
        let board = ChessBoard::from_str("k7/1q6/8/8/8/8/6Q1/5K2 w - - 0 1").unwrap();
        let mut board_move = mv!(PieceType::Queen, Square::G2, Square::B7);
        let next_board = board.make_move(board_move).unwrap();
        board_move.associate(&board, &next_board);

        assert_eq!(board_move.is_capture().unwrap(), true);

        let mut board_move = mv!(PieceType::Queen, Square::G2, Square::C6);
        let next_board = board.make_move(board_move).unwrap();
        board_move.associate(&board, &next_board);
        assert_eq!(board_move.is_capture().unwrap(), false);
    }
}
