use crate::errors::LibChessError as Error;
use crate::{BoardMove, ChessBoard, Color, MovePropertiesOnBoard};
use std::fmt;

const HISTORY_CAPACITY: usize = 80;

#[derive(Debug, Clone)]
pub struct GameHistory {
    positions: Vec<ChessBoard>,
    moves:     Vec<BoardMove>,
    metadata:  Vec<MovePropertiesOnBoard>,
}

impl Default for GameHistory {
    #[inline]
    fn default() -> Self {
        Self {
            positions: Vec::with_capacity(HISTORY_CAPACITY),
            moves:     Vec::with_capacity(HISTORY_CAPACITY),
            metadata:  Vec::with_capacity(HISTORY_CAPACITY),
        }
    }
}

impl fmt::Display for GameHistory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.positions.is_empty() {
            write!(f, "")
        } else {
            let mut game_history_string;
            let first_move_string = self.moves[0].to_string(self.metadata[0]);
            match self.positions[0].get_side_to_move() {
                Color::White => game_history_string = format!("1.{first_move_string} "),
                Color::Black => game_history_string = format!("1. ... {first_move_string}"),
            }

            let white_starting = self.positions[0].get_side_to_move() == Color::White;
            for i in 1..self.moves.len() {
                let mut next_move_string = self.moves[i].to_string(self.metadata[i]);
                next_move_string = if (i % 2 != 0) ^ white_starting {
                    format!("{}.{next_move_string} ", (i + 2) / 2)
                } else {
                    format!("{next_move_string} ")
                };
                game_history_string = format!("{game_history_string}{next_move_string}");
            }
            write!(f, "{game_history_string}")
        }
    }
}

impl GameHistory {
    pub fn from_position(position: ChessBoard) -> Self {
        let mut result = Self::default();
        result.positions.push(position);
        result
    }

    pub fn get_position_on_move(&self, move_number: usize) -> Result<ChessBoard, Error> {
        if move_number < self.positions.len() {
            Ok(self.positions[move_number])
        } else {
            Err(Error::WrongMoveNumber)
        }
    }

    pub fn get_last_position(&self) -> ChessBoard { self.positions.last().unwrap().clone() }

    pub fn push(&mut self, board_move: BoardMove, new_position: ChessBoard) -> &mut Self {
        self.metadata
            .push(MovePropertiesOnBoard::new(board_move, self.get_last_position()).unwrap());
        self.positions.push(new_position);
        self.moves.push(board_move);
        self
    }

    pub fn get_positions(&self) -> &Vec<ChessBoard> { &self.positions }

    pub fn get_moves(&self) -> &Vec<BoardMove> { &self.moves }

    pub fn get_metadata(&self) -> &Vec<MovePropertiesOnBoard> { &self.metadata }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use crate::{squares::*, PieceType::*};

    #[test]
    fn de_riviere_paul_morphy_1863() {
        let mut game = Game::default();
        let moves = vec![
            mv!(Pawn, E2, E4), // 1.
            mv!(Pawn, E7, E5),
            mv!(Knight, G1, F3), // 2.
            mv!(Knight, B8, C6),
            mv!(Bishop, F1, C4), // 3.
            mv!(Knight, G8, F6),
            mv!(Knight, F3, G5), // 4.
            mv!(Pawn, D7, D5),
            mv!(Pawn, E4, D5), // 5.
            mv!(Knight, C6, A5),
            mv!(Pawn, D2, D3), // 6.
            mv!(Pawn, H7, H6),
            mv!(Knight, G5, F3), // 7.
            mv!(Pawn, E5, E4),
            mv!(Queen, D1, E2), // 8.
            mv!(Knight, A5, C4),
            mv!(Pawn, D3, C4), // 9.
            mv!(Bishop, F8, C5),
            mv!(Pawn, H2, H3), // 10.
            castle_king_side!(),
            mv!(Knight, F3, H2), // 11.
            mv!(Knight, F6, H7),
            mv!(Knight, B1, D2), // 12.
            mv!(Pawn, F7, F5),
            mv!(Knight, D2, B3), // 13.
            mv!(Bishop, C5, D6),
            castle_king_side!(), // 14.
            mv!(Bishop, D6, H2),
            mv!(King, G1, H2), // 15.
            mv!(Pawn, F5, F4),
            mv!(Queen, E2, E4), // 16.
            mv!(Knight, H7, G5),
            mv!(Queen, E4, D4), // 17.
            mv!(Knight, G5, F3),
            mv!(Pawn, G2, F3), // 18.
            mv!(Queen, D8, H4),
            mv!(Rook, F1, H1), // 19.
            mv!(Bishop, C8, H3),
            mv!(Bishop, C1, D2), // 20.
            mv!(Rook, F8, F6),
        ];
        for m in moves.iter() {
            game.make_move(Action::MakeMove(*m)).unwrap();
        }
        game.make_move(Action::Resign).unwrap();

        println!("{}", game.get_position());
        println!("{}", game.get_action_history());
        assert_eq!(
            format!("{}", game.get_action_history()),
            String::from("1.e4 e5 2.Nf3 Nc6 3.Bc4 Nf6 4.Ng5 d5 5.exd5 Na5 6.d3 h6 7.Nf3 e4 8.Qe2 Nxc4 9.dxc4 Bc5 10.h3 O-O 11.Nh2 Nh7 12.Nd2 f5 13.Nb3 Bd6 14.O-O Bxh2+ 15.Kxh2 f4 16.Qxe4 Ng5 17.Qd4 Nf3+ 18.gxf3 Qh4 19.Rh1 Bxh3 20.Bd2 Rf6 ")
        );
    }
}
