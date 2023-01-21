use crate::errors::LibChessError as Error;
use std::fmt;
use std::str::FromStr;

pub const RANKS_NUMBER: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Rank {
    First,
    Second,
    Third,
    Fourth,
    Fifth,
    Sixth,
    Seventh,
    Eighth,
}

pub const RANKS: [Rank; 8] = [
    Rank::First,
    Rank::Second,
    Rank::Third,
    Rank::Fourth,
    Rank::Fifth,
    Rank::Sixth,
    Rank::Seventh,
    Rank::Eighth,
];

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Rank::First => "1",
                Rank::Second => "2",
                Rank::Third => "3",
                Rank::Fourth => "4",
                Rank::Fifth => "5",
                Rank::Sixth => "6",
                Rank::Seventh => "7",
                Rank::Eighth => "8",
            }
        )
    }
}

impl FromStr for Rank {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 1 {
            return Err(Error::InvalidBoardRankName);
        }

        match s.chars().next().unwrap() {
            '1' => Ok(Rank::First),
            '2' => Ok(Rank::Second),
            '3' => Ok(Rank::Third),
            '4' => Ok(Rank::Fourth),
            '5' => Ok(Rank::Fifth),
            '6' => Ok(Rank::Sixth),
            '7' => Ok(Rank::Seventh),
            '8' => Ok(Rank::Eighth),
            _ => Err(Error::InvalidBoardRankName),
        }
    }
}

impl Rank {
    #[inline]
    pub fn to_index(&self) -> usize { *self as usize }

    #[inline]
    pub fn from_index(n: usize) -> Result<Self, Error> {
        match n {
            0 => Ok(Rank::First),
            1 => Ok(Rank::Second),
            2 => Ok(Rank::Third),
            3 => Ok(Rank::Fourth),
            4 => Ok(Rank::Fifth),
            5 => Ok(Rank::Sixth),
            6 => Ok(Rank::Seventh),
            7 => Ok(Rank::Eighth),
            _ => Err(Error::InvalidBoardRankIndex { n }),
        }
    }

    #[inline]
    pub fn up(&self) -> Result<Self, Error> { Rank::from_index(self.to_index() + 1) }

    #[inline]
    pub fn down(&self) -> Result<Self, Error> {
        if self.to_index() == 0 {
            return Err(Error::NegativeBoardRankIndex);
        }
        Rank::from_index(self.to_index() - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_index_test() {
        assert_eq!(Rank::First.to_index(), 0);
    }

    #[test]
    fn from_index_test() {
        assert_eq!(Rank::from_index(5).unwrap(), Rank::Sixth);
    }

    #[test]
    fn from_index_test_fails() {
        assert!(Rank::from_index(10).is_err());
    }

    #[test]
    fn down_for_rank_fails() {
        assert!(Rank::First.down().is_err());
    }

    #[test]
    fn down_for_rank() {
        assert_eq!(Rank::Second.down().unwrap(), Rank::First);
    }

    #[test]
    fn up_for_rank_fails() {
        assert!(Rank::Eighth.up().is_err());
    }

    #[test]
    fn up_for_rank() {
        assert_eq!(Rank::Seventh.up().unwrap(), Rank::Eighth);
    }

    #[test]
    fn init_from_str() {
        assert_eq!(Rank::from_str("1").unwrap(), Rank::First);
    }

    #[test]
    fn init_from_str_fails() {
        assert!(Rank::from_str("123").is_err());
        assert!(Rank::from_str("0").is_err());
        assert!(Rank::from_str("9").is_err());
        assert!(Rank::from_str("-9").is_err());
        assert!(Rank::from_str("a").is_err());
    }
}
