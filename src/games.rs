//! This module implements the game of chess
//!
//! Rules of the game, terminating conditions and recording
//! the history of the game also implemented here  

use crate::boards::BoardMove;
use crate::boards::BLANK;
use crate::boards::{ChessBoard, LegalMoves};
use crate::colors::Color;
use crate::errors::{ChessBoardError, GameError};
use crate::game_history::GameHistory;
use crate::pieces::PieceType::*;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

const UNIQUE_POSITIONS_CAPACITY: usize = 100;

/// Represents available actions for the player
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    MakeMove(BoardMove),
    OfferDraw,
    AcceptDraw,
    DeclineDraw,
    Resign,
}

/// Represents the status of the game
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    Ongoing,
    DrawOffered,
    CheckMated(Color),
    Resigned(Color),
    FiftyMovesDrawDeclared,
    TheoreticalDrawDeclared,
    RepetitionDrawDeclared,
    DrawAccepted,
    Stalemate,
}

impl fmt::Display for GameStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let status_string = match self {
            GameStatus::Ongoing | GameStatus::DrawOffered => String::from("the game is ongoing"),
            GameStatus::CheckMated(color) => format!("{} won by checkmate", !*color),
            GameStatus::Resigned(color) => format!("{} won by resignation", !*color),
            GameStatus::DrawAccepted => String::from("draw declared by agreement"),
            GameStatus::FiftyMovesDrawDeclared => String::from("draw declared by a 50 moves rule"),
            GameStatus::TheoreticalDrawDeclared => String::from("draw: no enough pieces"),
            GameStatus::RepetitionDrawDeclared => {
                String::from("draw declared by repetition of moves")
            }
            GameStatus::Stalemate => String::from("stalemate"),
        };
        write!(f, "{}", status_string)
    }
}

/// The Game of Chess object
///
/// ## Examples
/// ```
/// use libchess::{Game, Action, GameStatus, Color};
/// use libchess::boards::{ChessBoard, Square, BoardMove, BoardMoveOption, PieceMove};
/// use libchess::{castle_king_side, castle_queen_side, mv, PieceType};
///
/// let mut game = Game::default();
/// let moves = vec![
///    mv!(Pawn, E2, E4),
///    mv!(Pawn, E7, E5),
///    mv!(Queen, D1, H5),
///    mv!(King, E8, E7),
///    mv!(Queen, H5, E5),
/// ];
///
/// for one in moves.iter() {
///     game.make_move(Action::MakeMove(*one)).unwrap();
/// }
/// assert_eq!(game.get_game_status(), GameStatus::CheckMated(Color::Black));
/// ```
#[derive(Debug, Clone)]
pub struct Game {
    position: ChessBoard,
    history: GameHistory,
    unique_positions_counter: HashMap<u64, usize>,
    status: GameStatus,
}

impl Default for Game {
    #[inline]
    fn default() -> Self {
        let board = ChessBoard::default();
        let mut result = Self {
            position: board.clone(),
            history: GameHistory::from_position(board),
            unique_positions_counter: HashMap::with_capacity(UNIQUE_POSITIONS_CAPACITY),
            status: GameStatus::Ongoing,
        };

        result.update_game_status(None).position_counter_increment();
        result
    }
}

impl Game {
    #[inline]
    pub fn from_board(board: ChessBoard) -> Self {
        let mut result = Self {
            position: board.clone(),
            history: GameHistory::from_position(board),
            unique_positions_counter: HashMap::with_capacity(UNIQUE_POSITIONS_CAPACITY),
            status: GameStatus::Ongoing,
        };

        result.update_game_status(None).position_counter_increment();
        result
    }

    #[inline]
    pub fn from_fen(fen: &str) -> Result<Self, ChessBoardError> {
        match ChessBoard::from_str(fen) {
            Ok(board) => Ok(Self::from_board(board)),
            Err(e) => Err(e),
        }
    }

    /// Returns the GameHistory object which represents a sequence of moves
    /// in PGN-like string
    #[inline]
    pub fn get_action_history(&self) -> &GameHistory {
        &self.history
    }

    /// Returns the current game position
    #[inline]
    pub fn get_position(&self) -> &ChessBoard {
        &self.position
    }

    /// Returns the game status. Only ``GameStatus::Ongoing`` and ``GameStatus::DrawOffered``
    /// are not terminal
    #[inline]
    pub fn get_game_status(&self) -> GameStatus {
        self.status
    }

    /// Returns the side to make move
    #[inline]
    pub fn get_side_to_move(&self) -> Color {
        self.get_position().get_side_to_move()
    }

    /// Returns number of times current position was arise
    #[inline]
    pub fn get_position_counter(&self, position: &ChessBoard) -> usize {
        match self.unique_positions_counter.get(&position.get_hash()) {
            Some(counter) => *counter,
            None => 0,
        }
    }

    /// Returns a set of legal moves in current position. Duplicates the
    /// functionality of the ``ChessBoard::get_legal_moves()``
    #[inline]
    pub fn get_legal_moves(&self) -> LegalMoves {
        self.position.get_legal_moves()
    }

    #[inline]
    fn set_game_status(&mut self, status: GameStatus) -> &mut Self {
        self.status = status;
        self
    }

    #[inline]
    fn set_position(&mut self, position: ChessBoard) {
        self.position = position;
    }

    #[inline]
    fn position_counter_increment(&mut self) -> &mut Self {
        self.unique_positions_counter.insert(
            self.get_position().get_hash(),
            self.get_position_counter(self.get_position()) + 1,
        );
        self
    }

    fn update_game_status(&mut self, last_action: Option<Action>) -> &mut Self {
        self.set_game_status(match last_action {
            None | Some(Action::MakeMove(_)) => {
                let position = self.get_position();
                if position.is_terminal() {
                    if position.get_check_mask().count_ones() > 0 {
                        GameStatus::CheckMated(self.get_side_to_move())
                    } else {
                        GameStatus::Stalemate
                    }
                } else if self.get_position_counter(position) == 3 {
                    GameStatus::RepetitionDrawDeclared
                } else if position.get_moves_since_capture() >= 100 {
                    GameStatus::FiftyMovesDrawDeclared
                } else if self.is_theoretical_draw_on_board() {
                    GameStatus::TheoreticalDrawDeclared
                } else {
                    GameStatus::Ongoing
                }
            }
            Some(Action::OfferDraw) => GameStatus::DrawOffered,
            Some(Action::DeclineDraw) => GameStatus::Ongoing,
            Some(Action::AcceptDraw) => GameStatus::DrawAccepted,
            Some(Action::Resign) => GameStatus::Resigned(self.get_side_to_move()),
        });

        if self.get_game_status() != GameStatus::Ongoing {
            println!("{}", self.get_game_status())
        }

        self
    }

    fn is_theoretical_draw_on_board(&self) -> bool {
        let white_pieces_number = self.position.get_color_mask(Color::White).count_ones();
        let black_pieces_number = self.position.get_color_mask(Color::Black).count_ones();

        if (white_pieces_number <= 2) & (black_pieces_number <= 2) {
            let bishops_and_knights = self.position.get_piece_type_mask(Knight)
                | self.position.get_piece_type_mask(Bishop);

            let white_can_not_checkmate = match white_pieces_number {
                1 => true,
                2 => self.position.get_color_mask(Color::White) & bishops_and_knights != BLANK,
                _ => unreachable!(),
            };
            let black_can_not_checkmate = match black_pieces_number {
                1 => true,
                2 => self.position.get_color_mask(Color::Black) & bishops_and_knights != BLANK,
                _ => unreachable!(),
            };
            if white_can_not_checkmate & black_can_not_checkmate {
                return true;
            }
        }

        false
    }

    /// The method to make moves during the game
    pub fn make_move(&mut self, action: Action) -> Result<&mut Self, GameError> {
        let game_status = self.get_game_status();
        if game_status == GameStatus::Ongoing {
            match action {
                Action::MakeMove(m) => match self.get_position().make_move(m) {
                    Ok(next_position) => {
                        self.set_position(next_position);
                        self.position_counter_increment();
                        self.history.push(m, self.position.clone());
                    }
                    Err(_) => {
                        return Err(GameError::IllegalActionDetected);
                    }
                },
                Action::AcceptDraw | Action::DeclineDraw => {
                    return Err(GameError::IllegalActionDetected);
                }
                Action::OfferDraw | Action::Resign => {}
            }
        } else if game_status == GameStatus::DrawOffered {
            match action {
                Action::MakeMove(_) | Action::OfferDraw => {
                    return Err(GameError::IllegalActionDetected);
                }
                Action::AcceptDraw | Action::DeclineDraw => {}
                Action::Resign => {}
            }
        } else {
            return Err(GameError::GameIsAlreadyFinished);
        }

        self.update_game_status(Some(action));
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boards::squares::*;
    use crate::boards::ZOBRIST_TABLES as ZOBRIST;
    use crate::boards::{BoardMove, BoardMoveOption, PieceMove};
    use crate::{castle_king_side, castle_queen_side, mv};

    #[test]
    fn simple_check_mate() {
        let mut game = Game::default();
        let moves = vec![
            mv!(Pawn, E2, E4),
            mv!(Pawn, E7, E5),
            mv!(Queen, D1, H5),
            mv!(King, E8, E7),
            mv!(Queen, H5, E5),
        ];
        for one in moves.iter() {
            game.make_move(Action::MakeMove(*one)).unwrap();
        }
        assert_eq!(game.get_game_status(), GameStatus::CheckMated(Color::Black));

        let mut game = Game::default();
        let moves = vec![
            mv!(Pawn, E2, E4),
            mv!(Pawn, E7, E5),
            mv!(Bishop, F1, C4),
            mv!(Knight, B8, C6),
            mv!(Queen, D1, F3),
            mv!(Knight, C6, D4),
            mv!(Queen, F3, F7),
        ];
        for one in moves.iter() {
            game.make_move(Action::MakeMove(*one)).unwrap();
        }
        assert_eq!(game.get_game_status(), GameStatus::CheckMated(Color::Black));
    }

    #[test]
    fn stalemate() {
        let mut game = Game::from_fen("3k4/3P4/4K3/8/8/8/8/8 w - - 0 1").unwrap();
        let moves = vec![mv!(King, E6, D6)];
        for one in moves.iter() {
            game.make_move(Action::MakeMove(*one)).unwrap();
        }
        assert_eq!(game.get_game_status(), GameStatus::Stalemate);
    }

    #[test]
    fn draw_declaration() {
        let mut game = Game::from_fen("8/8/8/p3k3/P7/4K3/8/8 w - - 0 1").unwrap();
        let moves = vec![
            mv!(King, E3, D3),
            mv!(King, E5, D5),
            mv!(King, D3, E3),
            mv!(King, D5, E5),
            mv!(King, E3, D3),
            mv!(King, E5, D5),
            mv!(King, D3, E3),
            mv!(King, D5, E5),
        ];
        for one in moves.iter() {
            game.make_move(Action::MakeMove(*one)).unwrap();
        }
        assert_eq!(game.get_game_status(), GameStatus::RepetitionDrawDeclared);
    }

    #[test]
    fn resignation() {
        let mut game = Game::default();
        game.make_move(Action::Resign).unwrap();
        assert_eq!(game.get_game_status(), GameStatus::Resigned(Color::White));
    }

    #[test]
    fn theoretical_draw() {
        let game = Game::from_fen("4k3/8/6b1/8/8/3NK3/8/8 w - - 0 1").unwrap();
        assert_eq!(game.get_game_status(), GameStatus::TheoreticalDrawDeclared);
    }

    #[test]
    fn albin_winawer_1896() {
        let mut game = Game::default();
        let moves = vec![
            mv!(Pawn, E2, E4), // 1.
            mv!(Pawn, E7, E5),
            mv!(Knight, G1, F3), // 2.
            mv!(Knight, B8, C6),
            mv!(Bishop, F1, C4), // 3.
            mv!(Bishop, F8, C5),
            mv!(Pawn, C2, C3), // 4.
            mv!(Knight, G8, F6),
            castle_king_side!(), // 5.
            mv!(Knight, F6, E4),
            mv!(Bishop, C4, D5), // 6.
            mv!(Knight, E4, F2),
            mv!(Rook, F1, F2), // 7.
            mv!(Bishop, C5, F2),
            mv!(King, G1, F2), // 8.
            mv!(Knight, C6, E7),
            mv!(Queen, D1, B3), // 9.
            castle_king_side!(),
            mv!(Bishop, D5, E4), // 10.
            mv!(Pawn, D7, D5),
            mv!(Bishop, E4, C2), // 11.
            mv!(Pawn, E5, E4),
            mv!(Knight, F3, E1), // 12.
            mv!(Knight, E7, G6),
            mv!(Pawn, C3, C4), // 13.
            mv!(Pawn, D5, D4),
            mv!(Queen, B3, G3), // 14.
            mv!(Pawn, F7, F5),
            mv!(King, F2, G1), // 15.
            mv!(Pawn, C7, C5),
            mv!(Pawn, D2, D3), // 16.
            mv!(Pawn, F5, F4),
            mv!(Queen, G3, F2), // 17.
            mv!(Pawn, E4, E3),
            mv!(Queen, F2, F3), // 18.
            mv!(Queen, D8, H4),
            mv!(Queen, F3, D5), // 19.
            mv!(King, G8, H8),
            mv!(Knight, E1, F3), // 20.
            mv!(Queen, H4, F2),
            mv!(King, G1, H1), // 21.
            mv!(Knight, G6, H4),
            mv!(Queen, D5, G5), // 22.
            mv!(Bishop, C8, H3),
        ];
        for one in moves.iter() {
            game.make_move(Action::MakeMove(*one)).unwrap();
        }
        game.make_move(Action::Resign).unwrap();

        assert_eq!(game.get_game_status(), GameStatus::Resigned(Color::White));
    }

    #[test]
    fn hashes() {
        let mut game = Game::default();
        let moves = vec![
            mv!(Pawn, E2, E4), // 1.
            mv!(Pawn, E7, E5),
            mv!(Knight, G1, F3), // 2.
            mv!(Knight, B8, C6),
            mv!(Bishop, F1, C4), // 3.
            mv!(Bishop, F8, C5),
            mv!(Pawn, C2, C3), // 4.
            mv!(Knight, G8, F6),
            castle_king_side!(), // 5.
            mv!(Knight, F6, E4),
            mv!(Bishop, C4, D5), // 6.
            mv!(Knight, E4, F2),
            mv!(Rook, F1, F2), // 7.
            mv!(Bishop, C5, F2),
            mv!(King, G1, F2), // 8.
            mv!(Knight, C6, E7),
            mv!(Queen, D1, B3), // 9.
            castle_king_side!(),
            mv!(Bishop, D5, E4), // 10.
            mv!(Pawn, D7, D5),
            mv!(Bishop, E4, C2), // 11.
            mv!(Pawn, E5, E4),
            mv!(Knight, F3, E1), // 12.
            mv!(Knight, E7, G6),
            mv!(Pawn, C3, C4), // 13.
            mv!(Pawn, D5, D4),
            mv!(Queen, B3, G3), // 14.
            mv!(Pawn, F7, F5),
            mv!(King, F2, G1), // 15.
            mv!(Pawn, C7, C5),
            mv!(Pawn, D2, D3), // 16.
            mv!(Pawn, F5, F4),
            mv!(Queen, G3, F2), // 17.
            mv!(Pawn, E4, E3),
            mv!(Queen, F2, F3), // 18.
            mv!(Queen, D8, H4),
            mv!(Queen, F3, D5), // 19.
            mv!(King, G8, H8),
            mv!(Knight, E1, F3), // 20.
            mv!(Queen, H4, F2),
            mv!(King, G1, H1), // 21.
            mv!(Knight, G6, H4),
            mv!(Queen, D5, G5), // 22.
            mv!(Bishop, C8, H3),
        ];
        for one in moves.iter() {
            game.make_move(Action::MakeMove(*one)).unwrap();
        }

        let direct_calculated_hash = ZOBRIST.calculate_position_hash(&game.get_position());
        let live_updating_hash = game.get_position().get_hash();
        assert_eq!(direct_calculated_hash, live_updating_hash);
    }
}
