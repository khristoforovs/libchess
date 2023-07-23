use crate::errors::LibChessError as Error;
use crate::Rank;
use std::fmt;
use std::ops::Not;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Color {
    White,
    Black,
}

pub const COLORS_NUMBER: usize = 2;

impl Not for Color {
    type Output = Color;

    #[inline]
    fn not(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display_str = match self {
            Color::White => "white",
            Color::Black => "black",
        };
        write!(f, "{display_str}")
    }
}

impl Color {
    #[inline]
    pub fn to_index(&self) -> usize { *self as usize }

    #[inline]
    pub fn from_index(n: usize) -> Result<Self, Error> {
        match n {
            0 => Ok(Color::White),
            1 => Ok(Color::Black),
            _ => Err(Error::InvalidColorIndex { n }),
        }
    }

    #[inline]
    pub fn get_back_rank(&self) -> Rank {
        match self {
            Color::White => Rank::First,
            Color::Black => Rank::Eighth,
        }
    }

    #[inline]
    pub fn get_promotion_rank(&self) -> Rank {
        match self {
            Color::White => Rank::Eighth,
            Color::Black => Rank::First,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_index_from_index() {
        for i in 0..COLORS_NUMBER {
            assert_eq!(Color::from_index(i).unwrap().to_index(), i);
        }
    }
}
