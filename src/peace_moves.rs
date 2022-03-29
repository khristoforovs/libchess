use crate::pieces::Piece;
use crate::square::Square;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PieceMove {
    piece: Piece,
    source: Square,
    destination: Square,
    promotion: Option<Piece>,
    captures: Option<bool>,
    status: Option<MoveStatus>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MoveStatus {
    Check,
    CheckMate,
    Neither,
}

impl fmt::Display for PieceMove {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let source_str = format!("{}{}", self.piece, self.source);
        let destination_str = match self.captures {
            Some(true) => format!("x{}", self.destination),
            _ => format!("{}", self.destination),
        };
        let promotion_str = match self.promotion {
            Some(x) => format!("={}", x),
            None => String::new(),
        };
        let status_str = match self.status {
            Some(MoveStatus::Check) => String::from("+"),
            Some(MoveStatus::CheckMate) => String::from("#"),
            _ => String::new(),
        };

        write!(
            f,
            "{}{}{}{}",
            source_str, destination_str, promotion_str, status_str,
        )
    }
}

impl PieceMove {
    #[inline]
    pub fn new(piece: Piece, source: Square, destination: Square) -> Self {
        Self {
            piece,
            source,
            destination,
            promotion: None,
            captures: None,
            status: None,
        }
    }

    #[inline]
    pub fn get_piece(&self) -> Piece {
        self.piece
    }

    #[inline]
    pub fn get_source_square(&self) -> Square {
        self.source
    }

    #[inline]
    pub fn get_destination_square(&self) -> Square {
        self.destination
    }

    #[inline]
    pub fn get_promotion(&self) -> Option<Piece> {
        self.promotion
    }

    #[inline]
    pub fn get_capture(&self) -> Option<bool> {
        self.captures
    }

    #[inline]
    pub fn get_status(&self) -> Option<MoveStatus> {
        self.status
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::colors::Color;

    #[test]
    fn san_format() {
        let mv = PieceMove::new(Piece::Pawn(Color::White), Square::E2, Square::E4);
        assert_eq!(format!("{}", mv), String::from("e2e4"));

        let mv = PieceMove::new(Piece::Rook(Color::Black), Square::A1, Square::A8);
        assert_eq!(format!("{}", mv), String::from("Ra1a8"));

        let mv = PieceMove {
            piece: Piece::Queen(Color::White),
            source: Square::E4,
            destination: Square::E8,
            captures: Some(true),
            promotion: None,
            status: Some(MoveStatus::CheckMate),
        };
        assert_eq!(format!("{}", mv), String::from("Qe4xe8#"));

        let mv = PieceMove {
            piece: Piece::Pawn(Color::Black),
            source: Square::A2,
            destination: Square::A1,
            captures: Some(false),
            promotion: Some(Piece::Queen(Color::Black)),
            status: Some(MoveStatus::Check),
        };
        assert_eq!(format!("{}", mv), String::from("a2a1=Q+"));
    }
}