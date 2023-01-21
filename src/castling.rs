use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CastlingRights {
    Neither,
    QueenSide,
    KingSide,
    BothSides,
}

pub const CASTLING_RIGHTS_NUMBER: usize = 4;

impl Add for CastlingRights {
    type Output = CastlingRights;

    fn add(self, other: CastlingRights) -> CastlingRights {
        let self_bits = self.to_bits();
        let other_bits = other.to_bits();
        CastlingRights::from_bits([self_bits[0] | other_bits[0], self_bits[1] | other_bits[1]])
    }
}

impl AddAssign for CastlingRights {
    fn add_assign(&mut self, other: Self) { *self = *self + other; }
}

impl Sub for CastlingRights {
    type Output = CastlingRights;

    fn sub(self, other: CastlingRights) -> CastlingRights {
        let self_bits = self.to_bits();
        let other_bits = other.to_bits();
        CastlingRights::from_bits([self_bits[0] & !other_bits[0], self_bits[1] & !other_bits[1]])
    }
}

impl SubAssign for CastlingRights {
    fn sub_assign(&mut self, other: Self) { *self = *self - other; }
}

impl fmt::Display for CastlingRights {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            CastlingRights::Neither => "",
            CastlingRights::QueenSide => "q",
            CastlingRights::KingSide => "k",
            CastlingRights::BothSides => "kq",
        };
        write!(f, "{s}")
    }
}

impl CastlingRights {
    #[inline]
    fn to_bits(self) -> [bool; 2] { [self.has_kingside(), self.has_queenside()] }

    #[inline]
    fn from_bits(bits: [bool; 2]) -> Self {
        #![allow(unreachable_patterns)]
        match bits {
            [false, false] => CastlingRights::Neither,
            [true, false] => CastlingRights::KingSide,
            [false, true] => CastlingRights::QueenSide,
            [true, true] => CastlingRights::BothSides,
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn to_index(self) -> usize { self as usize }

    #[inline]
    pub fn from_index(i: usize) -> CastlingRights {
        #![allow(unreachable_patterns)]
        match i {
            0 => CastlingRights::Neither,
            1 => CastlingRights::KingSide,
            2 => CastlingRights::QueenSide,
            3 => CastlingRights::BothSides,
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn has_kingside(&self) -> bool {
        matches!(self, CastlingRights::BothSides | CastlingRights::KingSide)
    }

    #[inline]
    pub fn has_queenside(&self) -> bool {
        matches!(self, CastlingRights::BothSides | CastlingRights::QueenSide)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn availability() {
        assert_eq!(CastlingRights::BothSides.has_kingside(), true);
        assert_eq!(CastlingRights::BothSides.has_queenside(), true);
        assert_eq!(CastlingRights::Neither.has_kingside(), false);
        assert_eq!(CastlingRights::Neither.has_queenside(), false);
        assert_eq!(CastlingRights::KingSide.has_kingside(), true);
        assert_eq!(CastlingRights::KingSide.has_queenside(), false);
        assert_eq!(CastlingRights::QueenSide.has_kingside(), false);
        assert_eq!(CastlingRights::QueenSide.has_queenside(), true);
    }

    #[test]
    fn adding() {
        assert_eq!(
            CastlingRights::Neither + CastlingRights::Neither,
            CastlingRights::Neither
        );
        assert_eq!(
            CastlingRights::Neither + CastlingRights::KingSide,
            CastlingRights::KingSide
        );
        assert_eq!(
            CastlingRights::Neither + CastlingRights::QueenSide,
            CastlingRights::QueenSide
        );
        assert_eq!(
            CastlingRights::Neither + CastlingRights::BothSides,
            CastlingRights::BothSides
        );
        assert_eq!(
            CastlingRights::KingSide + CastlingRights::KingSide,
            CastlingRights::KingSide
        );
        assert_eq!(
            CastlingRights::KingSide + CastlingRights::QueenSide,
            CastlingRights::BothSides
        );
        assert_eq!(
            CastlingRights::KingSide + CastlingRights::BothSides,
            CastlingRights::BothSides
        );
        assert_eq!(
            CastlingRights::BothSides + CastlingRights::QueenSide,
            CastlingRights::BothSides
        );
        assert_eq!(
            CastlingRights::BothSides + CastlingRights::BothSides,
            CastlingRights::BothSides
        );
    }

    #[test]
    fn subtracting() {
        assert_eq!(
            CastlingRights::Neither - CastlingRights::BothSides,
            CastlingRights::Neither
        );
        assert_eq!(
            CastlingRights::Neither - CastlingRights::KingSide,
            CastlingRights::Neither
        );
        assert_eq!(
            CastlingRights::KingSide - CastlingRights::KingSide,
            CastlingRights::Neither
        );
        assert_eq!(
            CastlingRights::KingSide - CastlingRights::QueenSide,
            CastlingRights::KingSide
        );
        assert_eq!(
            CastlingRights::KingSide - CastlingRights::Neither,
            CastlingRights::KingSide
        );
        assert_eq!(
            CastlingRights::BothSides - CastlingRights::QueenSide,
            CastlingRights::KingSide
        );
        assert_eq!(
            CastlingRights::BothSides - CastlingRights::KingSide,
            CastlingRights::QueenSide
        );
        assert_eq!(
            CastlingRights::BothSides - CastlingRights::BothSides,
            CastlingRights::Neither
        );
        assert_eq!(
            CastlingRights::BothSides - CastlingRights::Neither,
            CastlingRights::BothSides
        );
    }
}
