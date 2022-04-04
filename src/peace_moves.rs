use crate::pieces::PieceType;
use crate::square::Square;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PieceMove {
    piece: PieceType,
    source: Square,
    destination: Square,
    promotion: Option<PieceType>,
    captures: Option<bool>,
    status: Option<MoveStatus>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MoveStatus {
    Check,
    Checkmate,
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
            Some(MoveStatus::Checkmate) => String::from("#"),
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
    pub fn new(piece: PieceType, source: Square, destination: Square) -> Self {
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
    pub fn set_promotion(&mut self, promotion: Option<PieceType>) {
        self.promotion = promotion;
    }

    #[inline]
    pub fn set_capture(&mut self, captures: Option<bool>) {
        self.captures = captures;
    }

    #[inline]
    pub fn set_status(&mut self, status: Option<MoveStatus>) {
        self.status = status;
    }

    #[inline]
    pub fn get_piece(&self) -> PieceType {
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
    pub fn get_promotion(&self) -> Option<PieceType> {
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

    #[test]
    fn san_format() {
        let mv = PieceMove::new(PieceType::Pawn, Square::E2, Square::E4);
        assert_eq!(format!("{}", mv), String::from("Pe2e4"));

        let mv = PieceMove::new(PieceType::Rook, Square::A1, Square::A8);
        assert_eq!(format!("{}", mv), String::from("Ra1a8"));

        let mv = PieceMove {
            piece: PieceType::Queen,
            source: Square::E4,
            destination: Square::E8,
            captures: Some(true),
            promotion: None,
            status: Some(MoveStatus::Checkmate),
        };
        assert_eq!(format!("{}", mv), String::from("Qe4xe8#"));

        let mv = PieceMove {
            piece: PieceType::Pawn,
            source: Square::A2,
            destination: Square::A1,
            captures: Some(false),
            promotion: Some(PieceType::Queen),
            status: Some(MoveStatus::Check),
        };
        assert_eq!(format!("{}", mv), String::from("Pa2a1=Q+"));
    }

    #[test]
    fn set_properties() {
        let mut mv = PieceMove::new(PieceType::Pawn, Square::D7, Square::E8);
        assert_eq!(format!("{}", mv), String::from("Pd7e8"));

        mv.set_capture(Some(true));
        assert_eq!(format!("{}", mv), String::from("Pd7xe8"));

        mv.set_promotion(Some(PieceType::Queen));
        assert_eq!(format!("{}", mv), String::from("Pd7xe8=Q"));

        mv.set_status(Some(MoveStatus::Check));
        assert_eq!(format!("{}", mv), String::from("Pd7xe8=Q+"));
    }
}
