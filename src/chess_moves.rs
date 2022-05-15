use crate::boards::{BitBoard, ChessBoard, Square};
use crate::errors::ChessBoardError;
use crate::PieceType;
use std::fmt;

#[derive(Debug, Clone, Copy)]
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChessMove {
    MovePiece(PieceMove),
    CastleKingSide,
    CastleQueenSide,
}

impl fmt::Display for ChessMove {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ChessMove::MovePiece(m) => write!(f, "{}", m),
            ChessMove::CastleKingSide => write!(f, "O-O"),
            ChessMove::CastleQueenSide => write!(f, "O-O-O"),
        }
    }
}

impl ChessMove {
    pub fn piece_move(&self) -> Option<PieceMove> {
        match self {
            ChessMove::MovePiece(m) => Some(*m),
            ChessMove::CastleKingSide => None,
            ChessMove::CastleQueenSide => None,
        }
    }

    pub fn is_capture_on_board(&self, board: &ChessBoard) -> Result<bool, ChessBoardError> {
        if !board.is_legal_move(*self) {
            return Err(ChessBoardError::IllegalMoveDetected);
        }
        match self {
            ChessMove::MovePiece(m) => Ok(m.is_capture_on_board(board)),
            ChessMove::CastleKingSide | ChessMove::CastleQueenSide => Ok(false),
        }
    }
}

#[macro_export]
macro_rules! mv {
    ($piece_type:expr, $square_from:expr, $square_to:expr) => {
        ChessMove::MovePiece(PieceMove::new($piece_type, $square_from, $square_to, None))
    };

    ($piece_type:expr, $square_from:expr, $square_to:expr, $promotion:expr) => {
        ChessMove::MovePiece(PieceMove::new(
            $piece_type,
            $square_from,
            $square_to,
            Some($promotion),
        ))
    };
}

#[macro_export]
macro_rules! castle_king_side {
    () => {
        ChessMove::CastleKingSide
    };
}

#[macro_export]
macro_rules! castle_queen_side {
    () => {
        ChessMove::CastleQueenSide
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn move_representation() {
        let chess_move = mv!(PieceType::Pawn, Square::E2, Square::E4);
        assert_eq!(format!("{}", chess_move), "e2e4");

        let chess_move = mv!(
            PieceType::Pawn,
            Square::E7,
            Square::E8,
            PromotionPieceType::Queen
        );
        assert_eq!(format!("{}", chess_move), "e7e8=Q");

        let chess_move = mv!(PieceType::Queen, Square::A1, Square::A8);
        assert_eq!(format!("{}", chess_move), "Qa1a8");

        let chess_move = castle_queen_side!();
        assert_eq!(format!("{}", chess_move), "O-O-O");
    }

    #[test]
    fn capture() {
        let board = ChessBoard::from_str("k7/1q6/8/8/8/8/6Q1/5K2 w - - 0 1").unwrap();
        let chess_move = mv!(PieceType::Queen, Square::G2, Square::B7);
        assert_eq!(chess_move.is_capture_on_board(&board).unwrap(), true);

        let chess_move = mv!(PieceType::Queen, Square::G2, Square::C6);
        assert_eq!(chess_move.is_capture_on_board(&board).unwrap(), false);
    }
}
