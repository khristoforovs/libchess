use crate::board_files::{File, FILES};
use crate::board_ranks::{Rank, RANKS};
use crate::castling::CastlingRights;
use crate::colors::Color;
use crate::errors::Error;
use crate::pieces::{Piece, PieceType};
use crate::square::Square;
use std::fmt;
use std::ops::{Index, IndexMut};
use std::str;
use std::str::FromStr;

#[derive(Copy, Clone)]
pub struct BoardBuilder {
    pieces: [Option<Piece>; 64],
    side_to_move: Color,
    castle_rights: [CastlingRights; 2],
    en_passant: Option<Square>,
    moves_since_capture_counter: usize,
    black_moved_counter: usize,
}

impl Index<Square> for BoardBuilder {
    type Output = Option<Piece>;

    fn index(&self, index: Square) -> &Self::Output {
        &self.pieces[index.to_index()]
    }
}

impl IndexMut<Square> for BoardBuilder {
    fn index_mut(&mut self, index: Square) -> &mut Self::Output {
        &mut self.pieces[index.to_index()]
    }
}

impl Default for BoardBuilder {
    fn default() -> BoardBuilder {
        BoardBuilder::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }
}

impl FromStr for BoardBuilder {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut fen = BoardBuilder::new();
        let tokens: Vec<&str> = value.split(' ').collect();
        if (tokens.len() < 5) | (tokens.len() > 6) {
            return Err(Error::InvalidFENString {
                s: value.to_string(),
            });
        }

        let pieces = tokens[0];
        let side = tokens[1];
        let castles = tokens[2];
        let en_passant = if tokens.len() == 6 { tokens[3] } else { "" };
        fen.set_moves_since_capture_counter(match usize::from_str(tokens[tokens.len() - 2]) {
            Ok(c) => c,
            Err(_) => {
                return Err(Error::InvalidFENString {
                    s: value.to_string(),
                });
            }
        });
        fen.set_black_moves_counter(match usize::from_str(tokens[tokens.len() - 1]) {
            Ok(c) => c,
            Err(_) => {
                return Err(Error::InvalidFENString {
                    s: value.to_string(),
                });
            }
        });

        let mut current_rank = Rank::Eighth;
        let mut current_file = File::A;
        for c in pieces.chars() {
            match c {
                '/' => {
                    match current_rank.down() {
                        Ok(r) => current_rank = r,
                        Err(_) => {
                            return Err(Error::InvalidFENString {
                                s: value.to_string(),
                            });
                        }
                    };
                    current_file = File::A;
                }
                '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' => {
                    match File::from_index(current_file.to_index() + (c as usize) - ('0' as usize))
                    {
                        Ok(f) => current_file = f,
                        Err(_) => (),
                    }
                }
                'r' | 'R' | 'n' | 'N' | 'b' | 'B' | 'q' | 'Q' | 'k' | 'K' | 'p' | 'P' => {
                    let color = {
                        if c.is_uppercase() {
                            Color::White
                        } else {
                            Color::Black
                        }
                    };
                    let piece_type = match PieceType::from_str(c.to_string().as_str()) {
                        Ok(t) => t,
                        Err(_) => {
                            return Err(Error::InvalidFENString {
                                s: value.to_string(),
                            });
                        }
                    };
                    fen[Square::from_rank_file(current_rank, current_file)] =
                        Some(Piece(piece_type, color));
                    match current_file.right() {
                        Ok(f) => current_file = f,
                        Err(_) => (),
                    };
                }
                _ => {
                    return Err(Error::InvalidFENString {
                        s: value.to_string(),
                    });
                }
            }
        }

        match side {
            "w" | "W" => fen = *fen.set_side_to_move(Color::White),
            "b" | "B" => fen = *fen.set_side_to_move(Color::Black),
            _ => {
                return Err(Error::InvalidFENString {
                    s: value.to_string(),
                })
            }
        }

        if castles.contains("K") && castles.contains("Q") {
            fen.set_castling_rights(Color::White, CastlingRights::BothSides);
        } else if castles.contains("K") {
            fen.set_castling_rights(Color::White, CastlingRights::KingSide);
        } else if castles.contains("Q") {
            fen.set_castling_rights(Color::White, CastlingRights::QueenSide);
        } else {
            fen.set_castling_rights(Color::White, CastlingRights::Neither);
        }

        if castles.contains("k") && castles.contains("q") {
            fen.set_castling_rights(Color::Black, CastlingRights::BothSides);
        } else if castles.contains("k") {
            fen.set_castling_rights(Color::Black, CastlingRights::KingSide);
        } else if castles.contains("q") {
            fen.set_castling_rights(Color::Black, CastlingRights::QueenSide);
        } else {
            fen.set_castling_rights(Color::Black, CastlingRights::Neither);
        }

        if let Ok(sq) = Square::from_str(&en_passant) {
            fen.set_en_passant(Some(sq));
        }

        Ok(fen)
    }
}

impl fmt::Display for BoardBuilder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut pieces_string = String::new();
        let mut empty_squares: usize = 0;

        for rank in RANKS.iter().rev() {
            if *rank != Rank::Eighth {
                if empty_squares != 0 {
                    pieces_string += format!("{}", empty_squares).as_str();
                    empty_squares = 0;
                }
                pieces_string.push_str("/");
            }
            for file in FILES.iter() {
                match self[Square::from_rank_file(*rank, *file)] {
                    Some(p) => {
                        if empty_squares != 0 {
                            pieces_string += format!("{}", empty_squares).as_str();
                            empty_squares = 0;
                        }
                        let mut s = format!("{}", p.0);
                        match p.1 {
                            Color::White => s = s.to_uppercase(),
                            Color::Black => s = s.to_lowercase(),
                        }
                        pieces_string.push_str(&s);
                    }
                    None => {
                        empty_squares += 1;
                    }
                }
            }
        }

        let castles = format!(
            " {}{}",
            format!("{}", self.castle_rights[0]).to_uppercase(),
            self.castle_rights[1],
        );

        write!(
            f,
            "{} {}{} {} {} {}",
            pieces_string,
            match self.get_side_to_move() {
                Color::White => "w",
                Color::Black => "b",
            },
            castles,
            match self.en_passant {
                Some(value) => format!("{}", value),
                None => "-".to_string(),
            },
            self.get_moves_since_capture(),
            self.get_black_moved_counter(),
        )
    }
}

impl BoardBuilder {
    pub fn new() -> BoardBuilder {
        BoardBuilder {
            pieces: [None; 64],
            side_to_move: Color::White,
            castle_rights: [CastlingRights::Neither, CastlingRights::Neither],
            en_passant: None,
            moves_since_capture_counter: 0,
            black_moved_counter: 0,
        }
    }

    pub fn validate() -> Option<Error> {
        todo!()
    }

    pub fn setup<'a>(
        pieces: impl IntoIterator<Item = &'a (Square, PieceType, Color)>,
        side_to_move: Color,
        white_castle_rights: CastlingRights,
        black_castle_rights: CastlingRights,
        en_passant: Option<Square>,
        moves_since_capture_counter: usize,
        black_moved_counter: usize,
    ) -> BoardBuilder {
        let mut builder = BoardBuilder {
            pieces: [None; 64],
            side_to_move: side_to_move,
            castle_rights: [white_castle_rights, black_castle_rights],
            en_passant,
            moves_since_capture_counter,
            black_moved_counter,
        };

        for (s, p, c) in pieces.into_iter() {
            builder.pieces[s.to_index()] = Some(Piece(*p, *c));
        }

        builder
    }

    #[inline]
    pub fn get_pieces(&self) -> [Option<Piece>; 64] {
        self.pieces
    }

    #[inline]
    pub fn get_black_moved_counter(&self) -> usize {
        self.black_moved_counter
    }

    #[inline]
    pub fn get_moves_since_capture(&self) -> usize {
        self.moves_since_capture_counter
    }

    #[inline]
    pub fn get_castle_rights(&self, color: Color) -> CastlingRights {
        self.castle_rights[color.to_index()]
    }

    #[inline]
    pub fn get_side_to_move(&self) -> Color {
        self.side_to_move
    }

    pub fn set_black_moves_counter(&mut self, counter: usize) -> &mut Self {
        self.black_moved_counter = counter;
        self
    }

    pub fn set_moves_since_capture_counter(&mut self, counter: usize) -> &mut Self {
        self.moves_since_capture_counter = counter;
        self
    }

    pub fn set_side_to_move(&mut self, color: Color) -> &mut Self {
        self.side_to_move = color;
        self
    }

    pub fn set_castling_rights(&mut self, color: Color, rights: CastlingRights) -> &mut Self {
        self.castle_rights[color.to_index()] = rights;
        self
    }

    pub fn set_en_passant(&mut self, file: Option<Square>) -> *mut Self {
        self.en_passant = file;
        self
    }

    pub fn set_square<'a>(&'a mut self, square: Square, piece: Option<Piece>) -> &'a mut Self {
        self[square] = piece;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_from_string() {
        assert_eq!(
            format!("{}", BoardBuilder::default()),
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        );
        assert_eq!(
            format!(
                "{}",
                BoardBuilder::from_str(
                    "rnbq1bnr/pppkpppp/8/3p4/4P3/5N2/PPPP1PPP/RNBQKB1R w KQ - 2 3"
                )
                .unwrap()
            ),
            "rnbq1bnr/pppkpppp/8/3p4/4P3/5N2/PPPP1PPP/RNBQKB1R w KQ - 2 3"
        );
    }
}
