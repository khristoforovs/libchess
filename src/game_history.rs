use crate::{ChessBoard, ChessMove, Color, PieceType};
use std::fmt;

const HISTORY_CAPACITY: usize = 80;

#[derive(Debug, Clone, Copy)]
pub enum AmbiguityResolveType {
    ExtraFile,
    ExtraSquare,
    Neither,
}

#[derive(Debug, Clone, Copy)]
pub struct HistoryChessMove {
    chess_move: ChessMove,
    is_capture: bool,
    is_check: bool,
    is_checkmate: bool,
    ambiguity_type: AmbiguityResolveType,
}

impl HistoryChessMove {
    pub fn from_chess_move(
        chess_move: ChessMove,
        previous_board: &ChessBoard,
        current_board: &ChessBoard,
    ) -> Self {
        HistoryChessMove {
            chess_move,
            is_capture: match chess_move {
                ChessMove::MovePiece(m) => m.is_capture_on_board(previous_board),
                ChessMove::CastleKingSide => false,
                ChessMove::CastleQueenSide => false,
            },
            is_check: current_board.get_check_mask().count_ones() > 0,
            is_checkmate: current_board.get_legal_moves().len() == 0,
            ambiguity_type: {
                match chess_move {
                    ChessMove::MovePiece(m) => match m.get_piece_type() {
                        PieceType::King => AmbiguityResolveType::Neither,
                        _ => previous_board.get_move_ambiguity_type(m).unwrap(),
                    },
                    ChessMove::CastleKingSide => AmbiguityResolveType::Neither,
                    ChessMove::CastleQueenSide => AmbiguityResolveType::Neither,
                }
            },
        }
    }
}

impl fmt::Display for HistoryChessMove {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let check_string = if self.is_checkmate {
            String::from("#")
        } else if self.is_check {
            String::from("+")
        } else {
            String::new()
        };

        match self.chess_move {
            ChessMove::MovePiece(m) => {
                let piece_type_string = match m.get_piece_type() {
                    PieceType::Pawn => String::new(),
                    p => format!("{}", p),
                };
                let ambiguity_resolve_string = match self.ambiguity_type {
                    AmbiguityResolveType::ExtraFile => {
                        format!("{}", m.get_source_square().get_file())
                    }
                    AmbiguityResolveType::ExtraSquare => format!("{}", m.get_source_square()),
                    AmbiguityResolveType::Neither => String::new(),
                };
                let capture_string = if self.is_capture {
                    String::from("x")
                } else {
                    String::new()
                };
                let promotion_string = match m.get_promotion() {
                    Some(piece_type) => format!("={}", piece_type),
                    None => String::new(),
                };

                write!(
                    f,
                    "{}{}{}{}{}{}",
                    piece_type_string,
                    ambiguity_resolve_string,
                    capture_string,
                    m.get_destination_square(),
                    promotion_string,
                    check_string,
                )
            }
            ChessMove::CastleKingSide => write!(f, "O-O{}", check_string),
            ChessMove::CastleQueenSide => write!(f, "O-O-O{}", check_string),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GameHistory {
    positions: Vec<ChessBoard>,
    moves: Vec<HistoryChessMove>,
}

impl Default for GameHistory {
    #[inline]
    fn default() -> Self {
        Self {
            positions: Vec::with_capacity(HISTORY_CAPACITY),
            moves: Vec::with_capacity(HISTORY_CAPACITY),
        }
    }
}

impl fmt::Display for GameHistory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.positions.len() == 0 {
            write!(f, "")
        } else {
            let mut game_history_string = String::new();
            match self.positions[0].get_side_to_move() {
                Color::White => {
                    for (i, m) in self.get_moves().iter().enumerate() {
                        let next_move_string = if i % 2 == 0 {
                            format!("{}.{} ", (i + 2) / 2, m)
                        } else {
                            format!("{} ", m)
                        };
                        game_history_string =
                            format!("{}{}", game_history_string, next_move_string);
                    }
                }
                Color::Black => {
                    game_history_string = format!("1. ... {}", game_history_string);
                    for (i, m) in self.get_moves().iter().enumerate() {
                        let next_move_string = if i % 2 == 1 {
                            format!("{}.{} ", (i + 2) / 2 + 1, m)
                        } else {
                            format!("{} ", m)
                        };
                        game_history_string =
                            format!("{}{}", game_history_string, next_move_string);
                    }
                }
            }
            write!(f, "{}", game_history_string)
        }
    }
}

impl GameHistory {
    pub fn from_position(position: ChessBoard) -> Self {
        let mut result = Self::default();
        result.push_position(position);
        result
    }

    pub fn push(&mut self, chess_move: ChessMove, new_position: ChessBoard) -> &mut Self {
        self.push_position(new_position);
        self.push_move(chess_move);
        self
    }

    pub fn get_positions(&self) -> &Vec<ChessBoard> {
        &self.positions
    }

    pub fn get_moves(&self) -> &Vec<HistoryChessMove> {
        &self.moves
    }

    fn push_position(&mut self, position: ChessBoard) -> &mut Self {
        self.positions.push(position);
        self
    }

    fn push_move(&mut self, chess_move: ChessMove) -> &mut Self {
        let positions_seq_len = self.positions.len();
        let history_chess_move = HistoryChessMove::from_chess_move(
            chess_move,
            &self.positions[positions_seq_len - 2],
            &self.positions[positions_seq_len - 1],
        );
        self.moves.push(history_chess_move);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chess_moves::{ChessMove, PieceMove};
    use crate::games::{Action, Game};
    use crate::pieces::PieceType;
    use crate::squares::Square;

    #[test]
    fn de_riviere_paul_morphy_1863() {
        let mut game = Game::default();
        let moves = vec![
            mv!(PieceType::Pawn, Square::E2, Square::E4), // 1.
            mv!(PieceType::Pawn, Square::E7, Square::E5),
            mv!(PieceType::Knight, Square::G1, Square::F3), // 2.
            mv!(PieceType::Knight, Square::B8, Square::C6),
            mv!(PieceType::Bishop, Square::F1, Square::C4), // 3.
            mv!(PieceType::Knight, Square::G8, Square::F6),
            mv!(PieceType::Knight, Square::F3, Square::G5), // 4.
            mv!(PieceType::Pawn, Square::D7, Square::D5),
            mv!(PieceType::Pawn, Square::E4, Square::D5), // 5.
            mv!(PieceType::Knight, Square::C6, Square::A5),
            mv!(PieceType::Pawn, Square::D2, Square::D3), // 6.
            mv!(PieceType::Pawn, Square::H7, Square::H6),
            mv!(PieceType::Knight, Square::G5, Square::F3), // 7.
            mv!(PieceType::Pawn, Square::E5, Square::E4),
            mv!(PieceType::Queen, Square::D1, Square::E2), // 8.
            mv!(PieceType::Knight, Square::A5, Square::C4),
            mv!(PieceType::Pawn, Square::D3, Square::C4), // 9.
            mv!(PieceType::Bishop, Square::F8, Square::C5),
            mv!(PieceType::Pawn, Square::H2, Square::H3), // 10.
            castle_king_side!(),
            mv!(PieceType::Knight, Square::F3, Square::H2), // 11.
            mv!(PieceType::Knight, Square::F6, Square::H7),
            mv!(PieceType::Knight, Square::B1, Square::D2), // 12.
            mv!(PieceType::Pawn, Square::F7, Square::F5),
            mv!(PieceType::Knight, Square::D2, Square::B3), // 13.
            mv!(PieceType::Bishop, Square::C5, Square::D6),
            castle_king_side!(), // 14.
            mv!(PieceType::Bishop, Square::D6, Square::H2),
            mv!(PieceType::King, Square::G1, Square::H2), // 15.
            mv!(PieceType::Pawn, Square::F5, Square::F4),
            mv!(PieceType::Queen, Square::E2, Square::E4), // 16.
            mv!(PieceType::Knight, Square::H7, Square::G5),
            mv!(PieceType::Queen, Square::E4, Square::D4), // 17.
            mv!(PieceType::Knight, Square::G5, Square::F3),
            mv!(PieceType::Pawn, Square::G2, Square::F3), // 18.
            mv!(PieceType::Queen, Square::D8, Square::H4),
            mv!(PieceType::Rook, Square::F1, Square::H1), // 19.
            mv!(PieceType::Bishop, Square::C8, Square::H3),
            mv!(PieceType::Bishop, Square::C1, Square::D2), // 20.
            mv!(PieceType::Rook, Square::F8, Square::F6),
        ];
        for one in moves.iter() {
            game.make_move(Action::MakeMove(*one)).unwrap();
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
