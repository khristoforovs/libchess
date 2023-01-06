use crate::errors::PieceRepresentationError as Error;
use crate::Color;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieceType {
    King,
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Piece(pub PieceType, pub Color);

pub const PIECE_TYPES_NUMBER: usize = 6;

impl fmt::Display for PieceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                PieceType::Pawn => "P",
                PieceType::Knight => "N",
                PieceType::Bishop => "B",
                PieceType::Rook => "R",
                PieceType::Queen => "Q",
                PieceType::King => "K",
            }
        )
    }
}

impl FromStr for PieceType {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.len() > 1 {
            return Err(Error::InvalidPeaceRepresentation);
        }

        if value.is_empty() {
            return Ok(PieceType::Pawn);
        }
        match value.to_uppercase().as_str().chars().next().unwrap() {
            'P' => Ok(PieceType::Pawn),
            'N' => Ok(PieceType::Knight),
            'B' => Ok(PieceType::Bishop),
            'R' => Ok(PieceType::Rook),
            'Q' => Ok(PieceType::Queen),
            'K' => Ok(PieceType::King),
            _ => Err(Error::InvalidPeaceRepresentation),
        }
    }
}

impl PieceType {
    #[inline]
    pub fn to_index(&self) -> usize { *self as usize }

    pub fn from_index(n: usize) -> Result<Self, Error> {
        match n {
            0 => Ok(PieceType::Pawn),
            1 => Ok(PieceType::Knight),
            2 => Ok(PieceType::Bishop),
            3 => Ok(PieceType::Rook),
            4 => Ok(PieceType::Queen),
            5 => Ok(PieceType::King),
            _ => Err(Error::InvalidPeaceIndex { n }),
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
