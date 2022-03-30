use crate::colors::Color;
use crate::errors::Error;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Piece(PieceType, Color);

pub const NUMBER_PIECE_TYPES: usize = 6;

impl fmt::Display for PieceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                PieceType::Pawn => "",
                PieceType::Knight => "N",
                PieceType::Bishop => "B",
                PieceType::Rook => "R",
                PieceType::Queen => "Q",
                PieceType::King => "K",
            }
        )
    }
}

impl PieceType {
    #[inline]
    pub fn to_index(&self) -> usize {
        *self as usize
    }

    fn from_str(s: &str) -> Result<PieceType, Error> {
        if s.len() > 1 {
            return Err(Error::InvalidPeaceRepresentation);
        }

        if s.len() == 0 {
            return Ok(PieceType::Pawn);
        }
        match s.to_uppercase().as_str().chars().next().unwrap() {
            'N' => Ok(PieceType::Knight),
            'B' => Ok(PieceType::Bishop),
            'R' => Ok(PieceType::Rook),
            'Q' => Ok(PieceType::Queen),
            'K' => Ok(PieceType::King),
            _ => Err(Error::InvalidPeaceRepresentation),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_from_string() {
        assert_eq!(PieceType::from_str("").unwrap(), PieceType::Pawn);
        assert_eq!(PieceType::from_str("N").unwrap(), PieceType::Knight);
        assert_eq!(PieceType::from_str("Q").unwrap(), PieceType::Queen);
    }
}
