use crate::board_files::File;
use crate::board_ranks::Rank;
use crate::errors::{self, Error};
use std::fmt;
use std::str::FromStr;

pub const SQUARES_NUMBER: usize = 64;

#[derive(Default, Debug, Clone, Copy, PartialEq, Hash)]
pub struct Square(u8);

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            ((self.0 & 7) as u8 + b'a') as char,
            ((self.0 >> 3) as u8 + b'1') as char
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
    pub fn get_rank(&self) -> Rank {
        Rank::from_index((self.0 >> 3) as usize).unwrap()
    }

    #[inline]
    pub fn get_file(&self) -> File {
        File::from_index((self.0 & 7) as usize).unwrap()
    }

    #[inline]
    pub fn to_index(&self) -> usize {
        self.0 as usize
    }

    #[inline]
    pub fn to_int(&self) -> u8 {
        self.0
    }

    #[inline]
    pub fn up(&self) -> Result<Self, errors::Error> {
        Ok(Self::from_rank_file(self.get_rank().up()?, self.get_file()))
    }

    #[inline]
    pub fn down(&self) -> Result<Self, errors::Error> {
        Ok(Self::from_rank_file(
            self.get_rank().down()?,
            self.get_file(),
        ))
    }

    #[inline]
    pub fn left(&self) -> Result<Self, errors::Error> {
        Ok(Self::from_rank_file(
            self.get_rank(),
            self.get_file().left()?,
        ))
    }

    #[inline]
    pub fn right(&self) -> Result<Self, errors::Error> {
        Ok(Self::from_rank_file(
            self.get_rank(),
            self.get_file().right()?,
        ))
    }

    pub fn is_light(&self) -> bool {
        let rank_id = self.get_rank().to_index();
        let file_id = self.get_file().to_index();
        if (rank_id + file_id) % 2 == 0 {
            return false;
        }
        true
    }

    #[inline]
    pub fn is_dark(&self) -> bool {
        !self.is_light()
    }

    pub const A1: Square = Square(0);
    pub const B1: Square = Square(1);
    pub const C1: Square = Square(2);
    pub const D1: Square = Square(3);
    pub const E1: Square = Square(4);
    pub const F1: Square = Square(5);
    pub const G1: Square = Square(6);
    pub const H1: Square = Square(7);
    pub const A2: Square = Square(8);
    pub const B2: Square = Square(9);
    pub const C2: Square = Square(10);
    pub const D2: Square = Square(11);
    pub const E2: Square = Square(12);
    pub const F2: Square = Square(13);
    pub const G2: Square = Square(14);
    pub const H2: Square = Square(15);
    pub const A3: Square = Square(16);
    pub const B3: Square = Square(17);
    pub const C3: Square = Square(18);
    pub const D3: Square = Square(19);
    pub const E3: Square = Square(20);
    pub const F3: Square = Square(21);
    pub const G3: Square = Square(22);
    pub const H3: Square = Square(23);
    pub const A4: Square = Square(24);
    pub const B4: Square = Square(25);
    pub const C4: Square = Square(26);
    pub const D4: Square = Square(27);
    pub const E4: Square = Square(28);
    pub const F4: Square = Square(29);
    pub const G4: Square = Square(30);
    pub const H4: Square = Square(31);
    pub const A5: Square = Square(32);
    pub const B5: Square = Square(33);
    pub const C5: Square = Square(34);
    pub const D5: Square = Square(35);
    pub const E5: Square = Square(36);
    pub const F5: Square = Square(37);
    pub const G5: Square = Square(38);
    pub const H5: Square = Square(39);
    pub const A6: Square = Square(40);
    pub const B6: Square = Square(41);
    pub const C6: Square = Square(42);
    pub const D6: Square = Square(43);
    pub const E6: Square = Square(44);
    pub const F6: Square = Square(45);
    pub const G6: Square = Square(46);
    pub const H6: Square = Square(47);
    pub const A7: Square = Square(48);
    pub const B7: Square = Square(49);
    pub const C7: Square = Square(50);
    pub const D7: Square = Square(51);
    pub const E7: Square = Square(52);
    pub const F7: Square = Square(53);
    pub const G7: Square = Square(54);
    pub const H7: Square = Square(55);
    pub const A8: Square = Square(56);
    pub const B8: Square = Square(57);
    pub const C8: Square = Square(58);
    pub const D8: Square = Square(59);
    pub const E8: Square = Square(60);
    pub const F8: Square = Square(61);
    pub const G8: Square = Square(62);
    pub const H8: Square = Square(63);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_from_string() {
        assert_eq!(Square::from_str("e2").unwrap(), Square::E2);
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
        assert_eq!(Square::E4.up().unwrap(), Square::E5);
        assert_eq!(Square::E4.down().unwrap(), Square::E3);
        assert_eq!(Square::E4.left().unwrap(), Square::D4);
        assert_eq!(Square::E4.right().unwrap(), Square::F4);
    }

    #[test]
    fn test_light_dark() {
        assert_eq!(Square::A1.is_light(), false);
        assert_eq!(Square::E4.is_light(), true);
        assert_eq!(Square::A3.is_dark(), true);
        assert_eq!(Square::E6.is_dark(), false);
    }
}
