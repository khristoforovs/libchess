use crate::errors::LibChessError as Error;
use crate::{BitBoard, ChessBoard, PieceType, Square};
use std::fmt;
use std::hash::Hash;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DisplayAmbiguityType {
    ExtraFile,
    ExtraSquare,
    Neither,
}

#[derive(Debug, Clone, Copy)]
pub struct MovePropertiesOnBoard {
    pub is_check:       bool,
    pub is_checkmate:   bool,
    pub is_capture:     bool,
    pub ambiguity_type: DisplayAmbiguityType,
}

impl MovePropertiesOnBoard {
    pub fn new(board_move: BoardMove, board: ChessBoard) -> Result<Self, Error> {
        let board_after_move = board.make_move(board_move)?;
        let is_check = board_after_move.get_check_mask().count_ones() > 0;
        let is_checkmate = board_after_move.is_terminal() & is_check;
        let is_capture = match board_move {
            BoardMove::MovePiece(m) => m.is_capture_on_board(board),
            BoardMove::CastleKingSide => false,
            BoardMove::CastleQueenSide => false,
        };
        let ambiguity_type = match board_move {
            BoardMove::MovePiece(m) => match m.get_piece_type() {
                PieceType::King => DisplayAmbiguityType::Neither,
                _ => board.get_move_ambiguity_type(m)?,
            },
            BoardMove::CastleKingSide => DisplayAmbiguityType::Neither,
            BoardMove::CastleQueenSide => DisplayAmbiguityType::Neither,
        };

        Ok(Self {
            is_check,
            is_checkmate,
            is_capture,
            ambiguity_type,
        })
    }
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
            Some(piece_type) => format!("={piece_type}"),
            None => String::new(),
        };
        let piece_type_string = match self.get_piece_type() {
            PieceType::Pawn => String::new(),
            p => format!("{p}"),
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

impl FromStr for PieceMove {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let tokens: Vec<&str> = value.split('=').collect();
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

        PieceMove::new(
            piece_type,
            source_square,
            destination_square,
            if tokens.len() > 1 {
                Some(PieceType::from_str(tokens[1]).unwrap())
            } else {
                None
            },
        )
    }
}

impl PieceMove {
    pub fn new(
        piece_type: PieceType,
        square_from: Square,
        square_to: Square,
        promotion: Option<PieceType>,
    ) -> Result<Self, Error> {
        if promotion == Some(PieceType::Pawn) {
            return Err(Error::InvalidPromotionPiece);
        }
        Ok(PieceMove {
            piece_type,
            square_from,
            square_to,
            promotion,
        })
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
pub enum BoardMove {
    MovePiece(PieceMove),
    CastleKingSide,
    CastleQueenSide,
}

impl FromStr for BoardMove {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "O-O-O" => Ok(Self::CastleQueenSide),
            "O-O" => Ok(Self::CastleKingSide),
            s => Ok(Self::MovePiece(PieceMove::from_str(s)?)),
        }
    }
}

impl fmt::Display for BoardMove {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BoardMove::MovePiece(m) => write!(f, "{m}"),
            BoardMove::CastleKingSide => write!(f, "O-O"),
            BoardMove::CastleQueenSide => write!(f, "O-O-O"),
        }
    }
}

impl BoardMove {
    pub fn piece_move(&self) -> Result<PieceMove, Error> {
        if let BoardMove::MovePiece(m) = self {
            Ok(*m)
        } else {
            Err(Error::InvalidBoardMoveRepresentation)
        }
    }

    pub fn to_string(&self, properties: MovePropertiesOnBoard) -> String {
        let check_string = if properties.is_checkmate {
            "#"
        } else if properties.is_check {
            "+"
        } else {
            ""
        };

        match self {
            BoardMove::MovePiece(m) => {
                let piece_type_string = match m.get_piece_type() {
                    PieceType::Pawn => String::new(),
                    p => format!("{p}"),
                };
                let ambiguity_resolve_string = match properties.ambiguity_type {
                    DisplayAmbiguityType::ExtraFile => {
                        format!("{}", m.get_source_square().get_file())
                    }
                    DisplayAmbiguityType::ExtraSquare => format!("{}", m.get_source_square()),
                    DisplayAmbiguityType::Neither => String::new(),
                };
                let capture_string = if properties.is_capture { "x" } else { "" };
                let promotion_string = match m.get_promotion() {
                    Some(piece_type) => format!("={piece_type}"),
                    None => String::new(),
                };

                format!(
                    "{}{}{}{}{}{}",
                    piece_type_string,
                    ambiguity_resolve_string,
                    capture_string,
                    m.get_destination_square(),
                    promotion_string,
                    check_string,
                )
            }
            BoardMove::CastleKingSide => format!("O-O{check_string}"),
            BoardMove::CastleQueenSide => format!("O-O-O{check_string}"),
        }
    }
}

#[macro_export]
macro_rules! mv {
    ($piece_type:expr, $square_from:expr, $square_to:expr) => {
        BoardMove::MovePiece(PieceMove::new($piece_type, $square_from, $square_to, None).unwrap())
    };

    ($piece_type:expr, $square_from:expr, $square_to:expr, $promotion:expr) => {
        BoardMove::MovePiece(
            PieceMove::new($piece_type, $square_from, $square_to, Some($promotion)).unwrap(),
        )
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
        BoardMove::CastleKingSide
    };
}

#[macro_export]
macro_rules! castle_queen_side {
    () => {
        BoardMove::CastleQueenSide
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::squares::*;
    use std::str::FromStr;
    use PieceType::*;

    #[test]
    fn move_representation() {
        let board_move = mv!(Pawn, E2, E4);
        assert_eq!(format!("{}", board_move), "e2e4");

        let board_move = mv!(Pawn, E7, E8, Queen);
        assert_eq!(format!("{}", board_move), "e7e8=Q");

        let board_move = mv!(Queen, A1, A8);
        assert_eq!(format!("{}", board_move), "Qa1a8");

        let board_move = castle_queen_side!();
        assert_eq!(format!("{}", board_move), "O-O-O");
    }

    #[test]
    fn capture() {
        let board = ChessBoard::from_str("k7/1q6/8/8/8/8/6Q1/5K2 w - - 0 1").unwrap();
        let board_move = mv!(Queen, G2, B7);
        let metadata = MovePropertiesOnBoard::new(board_move, board).unwrap();
        assert_eq!(metadata.is_capture, true);

        let board_move = mv!(Queen, G2, C6);
        let metadata = MovePropertiesOnBoard::new(board_move, board).unwrap();
        assert_eq!(metadata.is_capture, false);
    }

    #[test]
    fn str_representation() {
        assert_eq!(BoardMove::from_str("e2e4").unwrap(), mv!(Pawn, E2, E4));

        assert_eq!(
            BoardMove::from_str("e7e8=Q").unwrap(),
            BoardMove::MovePiece(PieceMove::new(Pawn, E7, E8, Some(PieceType::Queen)).unwrap())
        );

        assert_eq!(
            BoardMove::from_str("Pe7e8=Q").unwrap(),
            BoardMove::MovePiece(PieceMove::new(Pawn, E7, E8, Some(PieceType::Queen)).unwrap())
        );

        assert_eq!(
            BoardMove::from_str("Ra1a8").unwrap(),
            BoardMove::MovePiece(PieceMove::new(Rook, A1, A8, None).unwrap())
        );

        assert_eq!(
            BoardMove::from_str("O-O-O").unwrap(),
            BoardMove::CastleQueenSide
        );

        assert_eq!(
            BoardMove::from_str("O-O").unwrap(),
            BoardMove::CastleKingSide
        );

        assert_eq!(
            BoardMove::from_str("bc1h6").unwrap(),
            BoardMove::MovePiece(PieceMove::new(Bishop, C1, H6, None).unwrap())
        );

        assert!(BoardMove::from_str("gc1h6").is_err());
        assert!(BoardMove::from_str("Bz1h6").is_err());
        assert!(BoardMove::from_str("Bc1h61").is_err());
    }
}
