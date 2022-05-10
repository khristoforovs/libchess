use crate::errors::ChessBoardCoordinatesError as Error;
use std::fmt;
use std::str::FromStr;

pub const FILES_NUMBER: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum File {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

pub const FILES: [File; 8] =
    [File::A, File::B, File::C, File::D, File::E, File::F, File::G, File::H];

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                File::A => "a",
                File::B => "b",
                File::C => "c",
                File::D => "d",
                File::E => "e",
                File::F => "f",
                File::G => "g",
                File::H => "h",
            }
        )
    }
}

impl FromStr for File {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 1 {
            return Err(Error::InvalidBoardFileName);
        }

        match s.chars().next().unwrap() {
            'a' => Ok(File::A),
            'b' => Ok(File::B),
            'c' => Ok(File::C),
            'd' => Ok(File::D),
            'e' => Ok(File::E),
            'f' => Ok(File::F),
            'g' => Ok(File::G),
            'h' => Ok(File::H),
            _ => Err(Error::InvalidBoardFileName),
        }
    }
}

impl File {
    #[inline]
    pub fn to_index(&self) -> usize {
        *self as usize
    }

    #[inline]
    pub fn from_index(n: usize) -> Result<Self, Error> {
        match n {
            0 => Ok(File::A),
            1 => Ok(File::B),
            2 => Ok(File::C),
            3 => Ok(File::D),
            4 => Ok(File::E),
            5 => Ok(File::F),
            6 => Ok(File::G),
            7 => Ok(File::H),
            _ => Err(Error::InvalidBoardFileIndex { n }),
        }
    }

    #[inline]
    pub fn right(&self) -> Result<Self, Error> {
        File::from_index(self.to_index() + 1)
    }

    #[inline]
    pub fn left(&self) -> Result<Self, Error> {
        if self.to_index() == 0 {
            return Err(Error::NegativeBoardFileIndex);
        }
        File::from_index(self.to_index() - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_index_test() {
        assert_eq!(File::A.to_index(), 0);
    }

    #[test]
    fn from_index_test() {
        assert_eq!(File::from_index(5).unwrap(), File::F);
    }

    #[test]
    fn from_index_test_fails() {
        assert!(File::from_index(10).is_err());
    }

    #[test]
    fn left_for_file_fails() {
        assert!(File::A.left().is_err());
    }

    #[test]
    fn left_for_file() {
        assert_eq!(File::B.left().unwrap(), File::A);
    }

    #[test]
    fn right_for_file_fails() {
        assert!(File::H.right().is_err());
    }

    #[test]
    fn right_for_file() {
        assert_eq!(File::G.right().unwrap(), File::H);
    }

    #[test]
    fn init_from_str() {
        assert_eq!(File::from_str("g").unwrap(), File::G);
    }

    #[test]
    fn init_from_str_fails() {
        assert!(File::from_str("bbb").is_err());
        assert!(File::from_str("0").is_err());
        assert!(File::from_str("9").is_err());
        assert!(File::from_str("-1").is_err());
    }
}
