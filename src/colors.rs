use std::ops::Not;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Color {
    White,
    Black,
}

pub const COLOR_NUMBER: usize = 2;

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

impl Color {
    #[inline]
    pub fn to_index(&self) -> usize {
        *self as usize
    }
}

