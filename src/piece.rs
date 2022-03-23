use crate::color::Color;
use crate::errors::Error;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Piece {
    Pawn(Color),
    Knight(Color),
    Bishop(Color),
    Rook(Color),
    Queen(Color),
    King(Color),
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Piece::Pawn(_) => "",
                Piece::Knight(_) => "N",
                Piece::Bishop(_) => "B",
                Piece::Rook(_) => "R",
                Piece::Queen(_) => "Q",
                Piece::King(_) => "K",
            }
        )
    }
}

impl Piece {
    pub fn color(&self) -> Color {
        match *self {
            Piece::Pawn(c) => c,
            Piece::Knight(c) => c,
            Piece::Bishop(c) => c,
            Piece::Rook(c) => c,
            Piece::Queen(c) => c,
            Piece::King(c) => c,
        }
    }

    fn from_str(s: &str, color: Color) -> Result<Piece, Error> {
        if s.len() > 1 { return Err(Error::InvalidPeaceRepresentation); }
    
        if s.len() == 0 { return Ok(Piece::Pawn(color)); }
        match s
            .to_uppercase()
            .as_str()
            .chars()
            .next()
            .unwrap()
        {
            'N' => Ok(Piece::Knight(color)),
            'B' => Ok(Piece::Bishop(color)),
            'R' => Ok(Piece::Rook(color)),
            'Q' => Ok(Piece::Queen(color)),
            'K' => Ok(Piece::King(color)),
            _ => Err(Error::InvalidPeaceRepresentation),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_from_string() {
        let color = Color::White;
        assert_eq!(Piece::from_str("", color).unwrap(), Piece::Pawn(color));
        println!("11");
        assert_eq!(Piece::from_str("N", color).unwrap(), Piece::Knight(color));
        assert_eq!(Piece::from_str("Q", color).unwrap(), Piece::Queen(color));
    }

    #[test]
    fn get_piece_color() {
        let color = Color::Black;
        assert_eq!(Piece::from_str("N", color).unwrap().color(), color);
    }
}