use crate::pieces::PieceType;
use crate::squares::Square;
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

#[derive(Debug, Clone, Copy)]
pub enum SourceSquareRepresentation {
    OnlyRank,
    OnlyFile,
    Full,
    None,
}

#[derive(Debug, Clone, Copy)]
pub struct ChessMove {
    piece_type: PieceType,
    square_from: Square,
    square_to: Square,
    promotion: Option<PieceType>,
    is_capture: bool,
    source_square_representation: SourceSquareRepresentation,
}

impl fmt::Display for ChessMove {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let promotion_string = match self.get_promotion() {
            Some(piece_type) => format!("->{}", piece_type),
            None => String::new(),
        };
        let source_square = self.get_source_square();
        let source_square_string = match self.get_source_square_representation() {
            SourceSquareRepresentation::OnlyRank => format!("{}", source_square.get_rank()),
            SourceSquareRepresentation::OnlyFile => format!("{}", source_square.get_file()),
            SourceSquareRepresentation::Full => format!("{}", source_square),
            SourceSquareRepresentation::None => String::new(),
        };
        let piece_type_string = match self.get_piece_type() {
            PieceType::Pawn => String::new(),
            p => format!("{}", p),
        };
        let capture_string = if self.get_capture() { "x" } else { "" };
        write!(
            f,
            "{}{}{}{}{}",
            piece_type_string,
            source_square_string,
            capture_string,
            self.get_destination_square(),
            promotion_string,
        )
    }
}

impl ChessMove {
    pub fn new(
        piece_type: PieceType,
        square_from: Square,
        square_to: Square,
        promotion: Option<PromotionPieceType>,
    ) -> Self {
        ChessMove {
            piece_type,
            square_from,
            square_to,
            promotion: {
                match promotion {
                    Some(p) => Some(PieceType::from(p)),
                    None => None,
                }
            },
            is_capture: false,
            source_square_representation: SourceSquareRepresentation::Full,
        }
    }

    pub fn get_piece_type(&self) -> PieceType {
        self.piece_type
    }

    pub fn get_source_square(&self) -> Square {
        self.square_from
    }

    pub fn get_destination_square(&self) -> Square {
        self.square_to
    }

    pub fn get_promotion(&self) -> Option<PieceType> {
        self.promotion
    }

    fn get_capture(&self) -> bool {
        self.is_capture
    }

    pub fn get_source_square_representation(&self) -> SourceSquareRepresentation {
        self.source_square_representation
    }

    pub fn set_source_square_representation(&mut self, representation: SourceSquareRepresentation) {
        self.source_square_representation = representation;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_representation() {
        let chess_move = ChessMove::new(PieceType::Pawn, Square::E2, Square::E4, None);
        assert_eq!(format!("{}", chess_move), "e2e4");

        let chess_move = ChessMove::new(
            PieceType::Pawn,
            Square::E7,
            Square::E8,
            Some(PromotionPieceType::Queen),
        );
        assert_eq!(format!("{}", chess_move), "e7e8->Q");

        let mut chess_move = ChessMove::new(
            PieceType::Pawn,
            Square::E7,
            Square::D8,
            Some(PromotionPieceType::Rook),
        );
        chess_move.is_capture = true;
        assert_eq!(format!("{}", chess_move), "e7xd8->R");

        let mut chess_move = ChessMove::new(PieceType::Queen, Square::A1, Square::A8, None);
        assert_eq!(format!("{}", chess_move), "Qa1a8");
        chess_move.set_source_square_representation(SourceSquareRepresentation::None);
        assert_eq!(format!("{}", chess_move), "Qa8");
        chess_move.set_source_square_representation(SourceSquareRepresentation::OnlyRank);
        assert_eq!(format!("{}", chess_move), "Q1a8");
        chess_move.set_source_square_representation(SourceSquareRepresentation::OnlyFile);
        assert_eq!(format!("{}", chess_move), "Qaa8");
    }
}
