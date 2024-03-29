use super::{File, Rank, Square, FILES, RANKS};
use std::fmt;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Mul, Not};

#[derive(PartialEq, Eq, PartialOrd, Clone, Copy, Default, Hash)]
pub struct BitBoard(u64);

pub const BLANK: BitBoard = BitBoard(0);

impl BitAnd for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn bitand(self, other: BitBoard) -> BitBoard { BitBoard(self.0 & other.0) }
}

impl BitOr for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn bitor(self, other: BitBoard) -> BitBoard { BitBoard(self.0 | other.0) }
}

impl BitXor for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn bitxor(self, other: BitBoard) -> BitBoard { BitBoard(self.0 ^ other.0) }
}

impl BitAndAssign for BitBoard {
    #[inline]
    fn bitand_assign(&mut self, other: BitBoard) { self.0 &= other.0; }
}

impl BitOrAssign for BitBoard {
    #[inline]
    fn bitor_assign(&mut self, other: BitBoard) { self.0 |= other.0; }
}

impl BitXorAssign for BitBoard {
    #[inline]
    fn bitxor_assign(&mut self, other: BitBoard) { self.0 ^= other.0; }
}

impl Mul for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn mul(self, other: BitBoard) -> BitBoard { BitBoard(self.0.wrapping_mul(other.0)) }
}

impl Not for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn not(self) -> BitBoard { BitBoard(!self.0) }
}

impl Iterator for BitBoard {
    type Item = Square;

    #[inline]
    fn next(&mut self) -> Option<Square> {
        if self.0 == 0 {
            None
        } else {
            let result = self.to_square();
            *self ^= BitBoard::from_square(result);
            Some(result)
        }
    }
}

impl fmt::Debug for BitBoard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "BitBoard({:#018x})", self.0) }
}

impl fmt::Display for BitBoard {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s: String = "".to_owned();
        for r in (0..8).rev() {
            for f in 0..8 {
                let n = r * 8 + f;
                if self.0 & (1u64 << n) == (1u64 << n) {
                    s.push_str("X ");
                } else {
                    s.push_str(". ");
                }
            }
            s.push('\n');
        }
        write!(f, "{s}")
    }
}

impl BitBoard {
    #[inline]
    pub fn new(b: u64) -> BitBoard { BitBoard(b) }

    pub fn bits(&self) -> u64 { self.0 }

    #[inline]
    pub fn from_square(square: Square) -> Self { Self::new(1u64 << square.to_int()) }

    #[inline]
    pub fn from_rank_file(rank: Rank, file: File) -> Self {
        Self::from_square(Square::from_rank_file(rank, file))
    }

    pub fn from_file(file: File) -> Self {
        let mut result = BLANK;
        for rank in RANKS.iter() {
            result ^= Self::from_square(Square::from_rank_file(*rank, file));
        }
        result
    }

    pub fn from_rank(rank: Rank) -> Self {
        let mut result = BLANK;
        for file in FILES.iter() {
            result ^= Self::from_square(Square::from_rank_file(rank, *file));
        }
        result
    }

    #[inline]
    pub fn count_ones(&self) -> u32 { self.0.count_ones() }

    #[inline]
    pub fn to_square(&self) -> Square { Square::new(self.0.trailing_zeros() as u8).unwrap() }

    #[inline]
    pub fn is_blank(&self) -> bool { self == &BLANK }

    #[inline]
    pub fn first_bit_square(&self) -> Option<Square> {
        if self.is_blank() {
            return None;
        }
        Some(Square::new(63 - self.0.leading_zeros() as u8).unwrap())
    }

    #[inline]
    pub fn last_bit_square(&self) -> Option<Square> {
        if self.is_blank() {
            return None;
        }
        Some(Square::new(self.0.trailing_zeros() as u8).unwrap())
    }
}

#[rustfmt::skip]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let bit_board = BitBoard::new(2);
        let result = 0x00000000000002u64;
        assert_eq!(bit_board.0, result);
    }

    #[test]
    fn create_from_rank_file() {
        let bit_board = BitBoard::from_rank_file(Rank::Second, File::E);
        println!("{}", bit_board);
        println!("{}", bit_board.to_square());
        assert_eq!(bit_board, bit_board);
    }

    #[test]
    fn bit_ops() {
        let bit_board = BitBoard::from_rank_file(Rank::Second, File::E)
            | BitBoard::from_rank_file(Rank::Fourth, File::E);
        let result = 0x0000000010001000u64;
        assert_eq!(bit_board.0, result);

        let bit_board = bit_board & BitBoard::from_rank_file(Rank::Fourth, File::E);
        let result = 0x0000000010000000u64;
        assert_eq!(bit_board.0, result);

        let bit_board = !bit_board;
        let result = 0xffffffffefffffffu64;
        assert_eq!(bit_board.0, result);
    }
}
