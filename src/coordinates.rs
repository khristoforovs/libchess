use super::File;
use super::Rank;
use crate::errors::LibChessError as Error;
use std::fmt;
use std::str::FromStr;

pub const SQUARES_NUMBER: usize = 64;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Square(u8);

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            ((self.0 & 7) + b'a') as char,
            ((self.0 >> 3) + b'1') as char
        )
    }
}

impl FromStr for Square {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 2 {
            return Err(Error::InvalidSquareRepresentation);
        }

        let chars: Vec<char> = s.chars().collect();
        let file = match File::from_str(&chars[0].to_string()[..]) {
            Ok(f) => f,
            Err(_) => return Err(Error::InvalidSquareRepresentation),
        };
        let rank = match Rank::from_str(&chars[1].to_string()[..]) {
            Ok(r) => r,
            Err(_) => return Err(Error::InvalidSquareRepresentation),
        };
        Ok(Square::from_rank_file(rank, file))
    }
}

impl Square {
    #[inline]
    pub fn new(square: u8) -> Result<Square, Error> {
        match square {
            0..=63 => Ok(Square(square)),
            _ => Err(Error::InvalidBoardFileName),
        }
    }

    #[inline]
    pub fn from_rank_file(rank: Rank, file: File) -> Square {
        Square((rank.to_index() as u8) << 3 ^ (file.to_index() as u8))
    }

    #[inline]
    pub fn get_rank(&self) -> Rank { Rank::from_index((self.0 >> 3) as usize).unwrap() }

    #[inline]
    pub fn get_file(&self) -> File { File::from_index((self.0 & 7) as usize).unwrap() }

    #[inline]
    pub fn offsets_from(&self, other: Square) -> (i32, i32) {
        (
            other.get_rank().to_index() as i32 - self.get_rank().to_index() as i32,
            other.get_file().to_index() as i32 - self.get_file().to_index() as i32,
        )
    }

    #[inline]
    pub fn to_index(&self) -> usize { self.0 as usize }

    #[inline]
    pub fn to_int(&self) -> u8 { self.0 }

    #[inline]
    pub fn up(&self) -> Result<Self, Error> {
        Ok(Self::from_rank_file(self.get_rank().up()?, self.get_file()))
    }

    #[inline]
    pub fn down(&self) -> Result<Self, Error> {
        Ok(Self::from_rank_file(
            self.get_rank().down()?,
            self.get_file(),
        ))
    }

    #[inline]
    pub fn left(&self) -> Result<Self, Error> {
        Ok(Self::from_rank_file(
            self.get_rank(),
            self.get_file().left()?,
        ))
    }

    #[inline]
    pub fn right(&self) -> Result<Self, Error> {
        Ok(Self::from_rank_file(
            self.get_rank(),
            self.get_file().right()?,
        ))
    }

    pub fn is_light(&self) -> bool {
        if (self.get_rank().to_index() + self.get_file().to_index()) % 2 == 0 {
            return false;
        }
        true
    }

    #[inline]
    pub fn is_dark(&self) -> bool { !self.is_light() }
}

macro_rules! define_square {
    ($square:ident, $index:literal) => {
        pub const $square: Square = Square($index);
    };
}

pub mod squares {
    use super::*;

    define_square!(A1, 0);
    define_square!(B1, 1);
    define_square!(C1, 2);
    define_square!(D1, 3);
    define_square!(E1, 4);
    define_square!(F1, 5);
    define_square!(G1, 6);
    define_square!(H1, 7);
    define_square!(A2, 8);
    define_square!(B2, 9);
    define_square!(C2, 10);
    define_square!(D2, 11);
    define_square!(E2, 12);
    define_square!(F2, 13);
    define_square!(G2, 14);
    define_square!(H2, 15);
    define_square!(A3, 16);
    define_square!(B3, 17);
    define_square!(C3, 18);
    define_square!(D3, 19);
    define_square!(E3, 20);
    define_square!(F3, 21);
    define_square!(G3, 22);
    define_square!(H3, 23);
    define_square!(A4, 24);
    define_square!(B4, 25);
    define_square!(C4, 26);
    define_square!(D4, 27);
    define_square!(E4, 28);
    define_square!(F4, 29);
    define_square!(G4, 30);
    define_square!(H4, 31);
    define_square!(A5, 32);
    define_square!(B5, 33);
    define_square!(C5, 34);
    define_square!(D5, 35);
    define_square!(E5, 36);
    define_square!(F5, 37);
    define_square!(G5, 38);
    define_square!(H5, 39);
    define_square!(A6, 40);
    define_square!(B6, 41);
    define_square!(C6, 42);
    define_square!(D6, 43);
    define_square!(E6, 44);
    define_square!(F6, 45);
    define_square!(G6, 46);
    define_square!(H6, 47);
    define_square!(A7, 48);
    define_square!(B7, 49);
    define_square!(C7, 50);
    define_square!(D7, 51);
    define_square!(E7, 52);
    define_square!(F7, 53);
    define_square!(G7, 54);
    define_square!(H7, 55);
    define_square!(A8, 56);
    define_square!(B8, 57);
    define_square!(C8, 58);
    define_square!(D8, 59);
    define_square!(E8, 60);
    define_square!(F8, 61);
    define_square!(G8, 62);
    define_square!(H8, 63);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_from_string() {
        assert_eq!(Square::from_str("e2").unwrap(), squares::E2);
    }

    #[test]
    fn create_from_string_fails() {
        assert!(Square::from_str("e20").is_err());
        assert!(Square::from_str("e9").is_err());
        assert!(Square::from_str("z4").is_err());
        assert!(Square::from_str("b0").is_err());
    }

    #[test]
    fn neighbor_squares() {
        assert_eq!(squares::E4.up().unwrap(), squares::E5);
        assert_eq!(squares::E4.down().unwrap(), squares::E3);
        assert_eq!(squares::E4.left().unwrap(), squares::D4);
        assert_eq!(squares::E4.right().unwrap(), squares::F4);
    }

    #[test]
    fn test_light_dark() {
        use squares::*;

        assert_eq!(A1.is_light(), false);
        assert_eq!(E4.is_light(), true);
        assert_eq!(A3.is_dark(), true);
        assert_eq!(E6.is_dark(), false);
    }

    #[test]
    fn test_offsets() {
        use squares::*;

        assert_eq!(A1.offsets_from(A8), (7, 0));
        assert_eq!(B8.offsets_from(B1), (-7, 0));
        assert_eq!(E3.offsets_from(D4), (1, -1));
    }
}
