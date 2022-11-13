use crate::boards::{BitBoard, ChessBoard, Square};
use crate::errors::BoardMoveRepresentationError as Error;
use crate::PieceType;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PromotionPieceType {
    Knight,
    Bishop,
    Rook,
    Queen,
}

impl FromStr for PromotionPieceType {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.len() != 1 {
            return Err(Error::InvalidBoardMoveRepresentation);
        }

        match value.to_uppercase().as_str().chars().next().unwrap() {
            'N' => Ok(PromotionPieceType::Knight),
            'B' => Ok(PromotionPieceType::Bishop),
            'R' => Ok(PromotionPieceType::Rook),
            'Q' => Ok(PromotionPieceType::Queen),
            _ => Err(Error::InvalidBoardMoveRepresentation),
        }
    }
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
    piece_type:  PieceType,
    square_from: Square,
    square_to:   Square,
    promotion:   Option<PieceType>,
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
            promotion: promotion.map(PieceType::from),
        }
    }

    #[inline]
    pub fn get_piece_type(&self) -> PieceType { self.piece_type }

    #[inline]
    pub fn get_source_square(&self) -> Square { self.square_from }

    #[inline]
    pub fn get_destination_square(&self) -> Square { self.square_to }

    #[inline]
    pub fn get_promotion(&self) -> Option<PieceType> { self.promotion }

    pub fn is_capture_on_board(&self, board: ChessBoard) -> bool {
        (BitBoard::from_square(self.get_destination_square())
            & board.get_color_mask(!board.get_side_to_move()))
        .count_ones()
            > 0
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

impl FromStr for BoardMove {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let tokens: Vec<&str> = value.split('=').collect();
        match value {
            "O-O-O" => Ok(Self::new(BoardMoveOption::CastleQueenSide)),
            "O-O" => Ok(Self::new(BoardMoveOption::CastleKingSide)),
            _ => {
                let piece_str = tokens[0];
                let len = piece_str.len();

                let piece_type = if len == 4 {
                    PieceType::Pawn
                } else {
                    match PieceType::from_str(&piece_str[..1]) {
                        Ok(p) => p,
                        Err(_) => {
                            return Err(Error::InvalidBoardMoveRepresentation);
                        }
                    }
                };

                let source_square = match Square::from_str(&piece_str[(len - 4)..(len - 2)]) {
                    Ok(s) => s,
                    Err(_) => {
                        return Err(Error::InvalidBoardMoveRepresentation);
                    }
                };
                let destination_square = match Square::from_str(&piece_str[(len - 2)..]) {
                    Ok(s) => s,
                    Err(_) => {
                        return Err(Error::InvalidBoardMoveRepresentation);
                    }
                };

                Ok(BoardMove::new(BoardMoveOption::MovePiece(PieceMove::new(
                    piece_type,
                    source_square,
                    destination_square,
                    if tokens.len() > 1 {
                        Some(PromotionPieceType::from_str(tokens[1]).unwrap())
                    } else {
                        None
                    },
                ))))
            }
        }
    }
}

impl fmt::Display for BoardMove {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let check_string = if Some(true) == self.is_checkmate() {
            "#"
        } else if Some(true) == self.is_check() {
            "+"
        } else {
            ""
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
                let capture_string = if Some(true) == self.is_capture() { "x" } else { "" };
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
    fn eq(&self, other: &Self) -> bool { self.board_move_option == other.board_move_option }
}

impl Hash for BoardMove {
    fn hash<H: Hasher>(&self, state: &mut H) { self.board_move_option.hash(state); }
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
    pub fn get_move_option(&self) -> BoardMoveOption { self.board_move_option }

    #[inline]
    pub fn is_capture(&self) -> Option<bool> { self.is_capture }

    #[inline]
    pub fn is_check(&self) -> Option<bool> { self.is_check }

    #[inline]
    pub fn is_checkmate(&self) -> Option<bool> { self.is_checkmate }

    pub fn associate(
        &mut self,
        board_before_move: ChessBoard,
        board_after_move: ChessBoard,
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
macro_rules! mv_str {
    ($board_move_str:expr) => {
        BoardMove::from_str($board_move_str).unwrap()
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
    use crate::boards::squares::*;
    use std::str::FromStr;
    use PieceType::*;

    #[test]
    fn move_representation() {
        let board_move = mv!(Pawn, E2, E4);
        assert_eq!(format!("{}", board_move), "e2e4");

        let board_move = mv!(Pawn, E7, E8, PromotionPieceType::Queen);
        assert_eq!(format!("{}", board_move), "e7e8=Q");

        let board_move = mv!(Queen, A1, A8);
        assert_eq!(format!("{}", board_move), "Qa1a8");

        let board_move = castle_queen_side!();
        assert_eq!(format!("{}", board_move), "O-O-O");
    }

    #[test]
    fn capture() {
        let board = ChessBoard::from_str("k7/1q6/8/8/8/8/6Q1/5K2 w - - 0 1").unwrap();
        let mut board_move = mv!(Queen, G2, B7);
        let next_board = board.make_move(board_move).unwrap();
        board_move.associate(board, next_board);

        assert_eq!(board_move.is_capture().unwrap(), true);

        let mut board_move = mv!(Queen, G2, C6);
        let next_board = board.make_move(board_move).unwrap();
        board_move.associate(board, next_board);
        assert_eq!(board_move.is_capture().unwrap(), false);
    }

    #[test]
    fn str_representation() {
        assert_eq!(
            BoardMove::from_str("e2e4").unwrap(),
            BoardMove::new(BoardMoveOption::MovePiece(PieceMove::new(
                Pawn, E2, E4, None
            )))
        );

        assert_eq!(
            BoardMove::from_str("e7e8=Q").unwrap(),
            BoardMove::new(BoardMoveOption::MovePiece(PieceMove::new(
                Pawn,
                E7,
                E8,
                Some(PromotionPieceType::Queen)
            )))
        );

        assert_eq!(
            BoardMove::from_str("Pe7e8=Q").unwrap(),
            BoardMove::new(BoardMoveOption::MovePiece(PieceMove::new(
                Pawn,
                E7,
                E8,
                Some(PromotionPieceType::Queen)
            )))
        );

        assert_eq!(
            BoardMove::from_str("Ra1a8").unwrap(),
            BoardMove::new(BoardMoveOption::MovePiece(PieceMove::new(
                Rook, A1, A8, None
            )))
        );

        assert_eq!(
            BoardMove::from_str("O-O-O").unwrap(),
            BoardMove::new(BoardMoveOption::CastleQueenSide)
        );

        assert_eq!(
            BoardMove::from_str("O-O").unwrap(),
            BoardMove::new(BoardMoveOption::CastleKingSide)
        );

        assert_eq!(
            BoardMove::from_str("bc1h6").unwrap(),
            BoardMove::new(BoardMoveOption::MovePiece(PieceMove::new(
                Bishop, C1, H6, None
            )))
        );

        assert!(BoardMove::from_str("gc1h6").is_err());
        assert!(BoardMove::from_str("Bz1h6").is_err());
        assert!(BoardMove::from_str("Bc1h61").is_err());
    }
}
