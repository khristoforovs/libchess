use super::{ChessBoard, File, Rank, Square, FILES, RANKS, SQUARES_NUMBER};
use crate::errors::LibChessError as Error;
use crate::{CastlingRights, Color, Piece, PieceType, COLORS_NUMBER};
use std::fmt;
use std::ops::{Index, IndexMut};
use std::str;
use std::str::FromStr;

/// The board builder is used for initializing the ChessBoard without position checks
///
/// It does not check the sanity of position, moves ordering etc.
/// Actually it just implements the parser of FEN strings and easy representation
/// of ChessBoard's inner parameters. There is no need to create this object
/// manually because ChessBoard implements initialization via BoardBuilder
/// under the hood.
/// Also this struct implements the default starting position of chess board
///
/// ## Examples
/// ```
/// use libchess::BoardBuilder;
/// use std::str::FromStr;
///
/// assert_eq!(
///     format!("{}", BoardBuilder::default()),
///     "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
/// );
///
/// let fen = "4k3/Q7/5K2/8/8/8/8/8 b - - 0 1";
/// println!("{:?}", BoardBuilder::from_str(fen).unwrap());
/// ```
#[derive(Debug, Copy, Clone)]
pub struct BoardBuilder {
    pieces: [Option<Piece>; SQUARES_NUMBER],
    side_to_move: Color,
    castle_rights: [CastlingRights; COLORS_NUMBER],
    en_passant: Option<Square>,
    moves_since_capture_or_pawn_move: usize,
    move_number: usize,
}

impl From<ChessBoard> for BoardBuilder {
    fn from(board: ChessBoard) -> Self {
        let mut pieces = vec![];
        for i in 0..SQUARES_NUMBER {
            let square = Square::new(i as u8).unwrap();
            if let Some(piece_type) = board.get_piece_type_on(square) {
                let color = board.get_piece_color_on(square).unwrap();
                pieces.push((square, Piece(piece_type, color)));
            }
        }

        BoardBuilder::setup(
            &pieces,
            board.get_side_to_move(),
            board.get_castle_rights(Color::White),
            board.get_castle_rights(Color::Black),
            board.get_en_passant(),
            board.get_moves_since_capture_or_pawn_move(),
            board.get_move_number(),
        )
    }
}

impl Index<Square> for BoardBuilder {
    type Output = Option<Piece>;

    fn index(&self, index: Square) -> &Self::Output { &self.pieces[index.to_index()] }
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
        if tokens.len() != 6 {
            return Err(Error::InvalidFENString {
                s: value.to_string(),
            });
        }

        let pieces = tokens[0];
        let side = tokens[1];
        let castles = tokens[2];
        let en_passant = tokens[3];
        fen.set_moves_since_capture_or_pawn_move(match usize::from_str(tokens[4]) {
            Ok(c) => c,
            Err(_) => {
                return Err(Error::InvalidFENString {
                    s: value.to_string(),
                });
            }
        });
        fen.set_move_number(match usize::from_str(tokens[5]) {
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
                    if let Ok(f) =
                        File::from_index(current_file.to_index() + (c as usize) - ('0' as usize))
                    {
                        current_file = f
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
                    if let Ok(f) = current_file.right() {
                        current_file = f
                    }
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

        if castles.contains('K') && castles.contains('Q') {
            fen.set_castling_rights(Color::White, CastlingRights::BothSides);
        } else if castles.contains('K') {
            fen.set_castling_rights(Color::White, CastlingRights::KingSide);
        } else if castles.contains('Q') {
            fen.set_castling_rights(Color::White, CastlingRights::QueenSide);
        } else {
            fen.set_castling_rights(Color::White, CastlingRights::Neither);
        }

        if castles.contains('k') && castles.contains('q') {
            fen.set_castling_rights(Color::Black, CastlingRights::BothSides);
        } else if castles.contains('k') {
            fen.set_castling_rights(Color::Black, CastlingRights::KingSide);
        } else if castles.contains('q') {
            fen.set_castling_rights(Color::Black, CastlingRights::QueenSide);
        } else {
            fen.set_castling_rights(Color::Black, CastlingRights::Neither);
        }

        if let Ok(sq) = Square::from_str(en_passant) {
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
                pieces_string.push('/');
            }
            for file in FILES.iter() {
                match self[Square::from_rank_file(*rank, *file)] {
                    Some(p) => {
                        if empty_squares != 0 {
                            pieces_string += format!("{empty_squares}").as_str();
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
            if empty_squares != 0 {
                pieces_string += format!("{empty_squares}").as_str();
                empty_squares = 0;
            }
        }

        let castles_string = match self.castle_rights {
            [CastlingRights::Neither, CastlingRights::Neither] => "-".to_string(),
            _ => {
                format!(
                    "{}{}",
                    format!("{}", self.castle_rights[0]).to_uppercase(),
                    self.castle_rights[1]
                )
            }
        };

        write!(
            f,
            "{} {} {} {} {} {}",
            pieces_string,
            match self.get_side_to_move() {
                Color::White => "w",
                Color::Black => "b",
            },
            castles_string,
            match self.en_passant {
                Some(value) => format!("{value}"),
                None => "-".to_string(),
            },
            self.get_moves_since_capture_or_pawn_move(),
            self.get_move_number(),
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
            moves_since_capture_or_pawn_move: 0,
            move_number: 0,
        }
    }

    pub fn setup<'a>(
        pieces: impl IntoIterator<Item = &'a (Square, Piece)>,
        side_to_move: Color,
        white_castle_rights: CastlingRights,
        black_castle_rights: CastlingRights,
        en_passant: Option<Square>,
        moves_since_capture_or_pawn_move: usize,
        move_number: usize,
    ) -> BoardBuilder {
        let mut builder = BoardBuilder {
            pieces: [None; SQUARES_NUMBER],
            side_to_move,
            castle_rights: [white_castle_rights, black_castle_rights],
            en_passant,
            moves_since_capture_or_pawn_move,
            move_number,
        };

        for (s, p) in pieces.into_iter() {
            builder.pieces[s.to_index()] = Some(*p);
        }

        builder
    }

    #[inline]
    pub fn get_pieces(&self) -> [Option<Piece>; 64] { self.pieces }

    #[inline]
    pub fn get_move_number(&self) -> usize { self.move_number }

    #[inline]
    pub fn get_moves_since_capture_or_pawn_move(&self) -> usize {
        self.moves_since_capture_or_pawn_move
    }

    #[inline]
    pub fn get_castle_rights(&self, color: Color) -> CastlingRights {
        self.castle_rights[color.to_index()]
    }

    #[inline]
    pub fn get_side_to_move(&self) -> Color { self.side_to_move }

    #[inline]
    pub fn get_en_passant(&self) -> Option<Square> { self.en_passant }

    pub fn set_move_number(&mut self, counter: usize) -> &mut Self {
        self.move_number = counter;
        self
    }

    pub fn set_moves_since_capture_or_pawn_move(&mut self, counter: usize) -> &mut Self {
        self.moves_since_capture_or_pawn_move = counter;
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

    pub fn set_en_passant(&mut self, square: Option<Square>) -> &mut Self {
        self.en_passant = square;
        self
    }

    pub fn set_square(&mut self, square: Square, piece: Option<Piece>) -> &mut Self {
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

        let fen = "rnbq1bnr/pppkpppp/8/3p4/4P3/5N2/PPPP1PPP/RNBQKB1R w KQ - 2 3";
        assert_eq!(format!("{}", BoardBuilder::from_str(fen).unwrap()), fen);

        let fen = "8/8/5k2/8/3Q2N1/5K2/8/8 b - - 0 1";
        assert_eq!(format!("{}", BoardBuilder::from_str(fen).unwrap()), fen);

        let fen = "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 1";
        assert_eq!(format!("{}", BoardBuilder::from_str(fen).unwrap()), fen);
    }
}
