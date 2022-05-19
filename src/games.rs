//! This module implements the game of chess
//!
//! Rules of the game, terminating conditions and recording
//! the history of the game also implemented here  

use crate::boards::BLANK;
use crate::boards::{ChessBoard, LegalMoves};
use crate::chess_moves::ChessMove;
use crate::colors::Color;
use crate::errors::{ChessBoardError, GameError};
use crate::game_history::GameHistory;
use crate::pieces::PieceType;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

const UNIQUE_POSITIONS_CAPACITY: usize = 100;

/// Represents available actions for the player
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    MakeMove(ChessMove),
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
/// use libchess::{mv, PieceType, PieceMove, ChessMove};
/// use libchess::boards::Square;
///
/// let mut game = Game::default();
/// let moves = vec![
///    mv!(PieceType::Pawn, Square::E2, Square::E4),
///    mv!(PieceType::Pawn, Square::E7, Square::E5),
///    mv!(PieceType::Queen, Square::D1, Square::H5),
///    mv!(PieceType::King, Square::E8, Square::E7),
///    mv!(PieceType::Queen, Square::H5, Square::E5),
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
        match ChessBoard::from_str(&fen) {
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
                } else {
                    if self.get_position_counter(position) == 3 {
                        GameStatus::RepetitionDrawDeclared
                    } else if position.get_moves_since_capture() >= 100 {
                        GameStatus::FiftyMovesDrawDeclared
                    } else if self.is_theoretical_draw_on_board() {
                        GameStatus::TheoreticalDrawDeclared
                    } else {
                        GameStatus::Ongoing
                    }
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
            let bishops_and_knights = self.position.get_piece_type_mask(PieceType::Knight)
                | self.position.get_piece_type_mask(PieceType::Bishop);

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
    use crate::boards::Square;
    use crate::boards::ZOBRIST_TABLES as ZOBRIST;
    use crate::chess_moves::PieceMove;

    use super::*;

    #[test]
    fn simple_check_mate() {
        let mut game = Game::default();
        let moves = vec![
            mv!(PieceType::Pawn, Square::E2, Square::E4),
            mv!(PieceType::Pawn, Square::E7, Square::E5),
            mv!(PieceType::Queen, Square::D1, Square::H5),
            mv!(PieceType::King, Square::E8, Square::E7),
            mv!(PieceType::Queen, Square::H5, Square::E5),
        ];
        for one in moves.iter() {
            game.make_move(Action::MakeMove(*one)).unwrap();
        }
        assert_eq!(game.get_game_status(), GameStatus::CheckMated(Color::Black));

        let mut game = Game::default();
        let moves = vec![
            mv!(PieceType::Pawn, Square::E2, Square::E4),
            mv!(PieceType::Pawn, Square::E7, Square::E5),
            mv!(PieceType::Bishop, Square::F1, Square::C4),
            mv!(PieceType::Knight, Square::B8, Square::C6),
            mv!(PieceType::Queen, Square::D1, Square::F3),
            mv!(PieceType::Knight, Square::C6, Square::D4),
            mv!(PieceType::Queen, Square::F3, Square::F7),
        ];
        for one in moves.iter() {
            game.make_move(Action::MakeMove(*one)).unwrap();
        }
        assert_eq!(game.get_game_status(), GameStatus::CheckMated(Color::Black));
    }

    #[test]
    fn stalemate() {
        let mut game = Game::from_fen("3k4/3P4/4K3/8/8/8/8/8 w - - 0 1").unwrap();
        let moves = vec![mv!(PieceType::King, Square::E6, Square::D6)];
        for one in moves.iter() {
            game.make_move(Action::MakeMove(*one)).unwrap();
        }
        assert_eq!(game.get_game_status(), GameStatus::Stalemate);
    }

    #[test]
    fn draw_declaration() {
        let mut game = Game::from_fen("8/8/8/p3k3/P7/4K3/8/8 w - - 0 1").unwrap();
        let moves = vec![
            mv!(PieceType::King, Square::E3, Square::D3),
            mv!(PieceType::King, Square::E5, Square::D5),
            mv!(PieceType::King, Square::D3, Square::E3),
            mv!(PieceType::King, Square::D5, Square::E5),
            mv!(PieceType::King, Square::E3, Square::D3),
            mv!(PieceType::King, Square::E5, Square::D5),
            mv!(PieceType::King, Square::D3, Square::E3),
            mv!(PieceType::King, Square::D5, Square::E5),
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
            mv!(PieceType::Pawn, Square::E2, Square::E4), // 1.
            mv!(PieceType::Pawn, Square::E7, Square::E5),
            mv!(PieceType::Knight, Square::G1, Square::F3), // 2.
            mv!(PieceType::Knight, Square::B8, Square::C6),
            mv!(PieceType::Bishop, Square::F1, Square::C4), // 3.
            mv!(PieceType::Bishop, Square::F8, Square::C5),
            mv!(PieceType::Pawn, Square::C2, Square::C3), // 4.
            mv!(PieceType::Knight, Square::G8, Square::F6),
            castle_king_side!(), // 5.
            mv!(PieceType::Knight, Square::F6, Square::E4),
            mv!(PieceType::Bishop, Square::C4, Square::D5), // 6.
            mv!(PieceType::Knight, Square::E4, Square::F2),
            mv!(PieceType::Rook, Square::F1, Square::F2), // 7.
            mv!(PieceType::Bishop, Square::C5, Square::F2),
            mv!(PieceType::King, Square::G1, Square::F2), // 8.
            mv!(PieceType::Knight, Square::C6, Square::E7),
            mv!(PieceType::Queen, Square::D1, Square::B3), // 9.
            castle_king_side!(),
            mv!(PieceType::Bishop, Square::D5, Square::E4), // 10.
            mv!(PieceType::Pawn, Square::D7, Square::D5),
            mv!(PieceType::Bishop, Square::E4, Square::C2), // 11.
            mv!(PieceType::Pawn, Square::E5, Square::E4),
            mv!(PieceType::Knight, Square::F3, Square::E1), // 12.
            mv!(PieceType::Knight, Square::E7, Square::G6),
            mv!(PieceType::Pawn, Square::C3, Square::C4), // 13.
            mv!(PieceType::Pawn, Square::D5, Square::D4),
            mv!(PieceType::Queen, Square::B3, Square::G3), // 14.
            mv!(PieceType::Pawn, Square::F7, Square::F5),
            mv!(PieceType::King, Square::F2, Square::G1), // 15.
            mv!(PieceType::Pawn, Square::C7, Square::C5),
            mv!(PieceType::Pawn, Square::D2, Square::D3), // 16.
            mv!(PieceType::Pawn, Square::F5, Square::F4),
            mv!(PieceType::Queen, Square::G3, Square::F2), // 17.
            mv!(PieceType::Pawn, Square::E4, Square::E3),
            mv!(PieceType::Queen, Square::F2, Square::F3), // 18.
            mv!(PieceType::Queen, Square::D8, Square::H4),
            mv!(PieceType::Queen, Square::F3, Square::D5), // 19.
            mv!(PieceType::King, Square::G8, Square::H8),
            mv!(PieceType::Knight, Square::E1, Square::F3), // 20.
            mv!(PieceType::Queen, Square::H4, Square::F2),
            mv!(PieceType::King, Square::G1, Square::H1), // 21.
            mv!(PieceType::Knight, Square::G6, Square::H4),
            mv!(PieceType::Queen, Square::D5, Square::G5), // 22.
            mv!(PieceType::Bishop, Square::C8, Square::H3),
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
            mv!(PieceType::Pawn, Square::E2, Square::E4), // 1.
            mv!(PieceType::Pawn, Square::E7, Square::E5),
            mv!(PieceType::Knight, Square::G1, Square::F3), // 2.
            mv!(PieceType::Knight, Square::B8, Square::C6),
            mv!(PieceType::Bishop, Square::F1, Square::C4), // 3.
            mv!(PieceType::Bishop, Square::F8, Square::C5),
            mv!(PieceType::Pawn, Square::C2, Square::C3), // 4.
            mv!(PieceType::Knight, Square::G8, Square::F6),
            castle_king_side!(), // 5.
            mv!(PieceType::Knight, Square::F6, Square::E4),
            mv!(PieceType::Bishop, Square::C4, Square::D5), // 6.
            mv!(PieceType::Knight, Square::E4, Square::F2),
            mv!(PieceType::Rook, Square::F1, Square::F2), // 7.
            mv!(PieceType::Bishop, Square::C5, Square::F2),
            mv!(PieceType::King, Square::G1, Square::F2), // 8.
            mv!(PieceType::Knight, Square::C6, Square::E7),
            mv!(PieceType::Queen, Square::D1, Square::B3), // 9.
            castle_king_side!(),
            mv!(PieceType::Bishop, Square::D5, Square::E4), // 10.
            mv!(PieceType::Pawn, Square::D7, Square::D5),
            mv!(PieceType::Bishop, Square::E4, Square::C2), // 11.
            mv!(PieceType::Pawn, Square::E5, Square::E4),
            mv!(PieceType::Knight, Square::F3, Square::E1), // 12.
            mv!(PieceType::Knight, Square::E7, Square::G6),
            mv!(PieceType::Pawn, Square::C3, Square::C4), // 13.
            mv!(PieceType::Pawn, Square::D5, Square::D4),
            mv!(PieceType::Queen, Square::B3, Square::G3), // 14.
            mv!(PieceType::Pawn, Square::F7, Square::F5),
            mv!(PieceType::King, Square::F2, Square::G1), // 15.
            mv!(PieceType::Pawn, Square::C7, Square::C5),
            mv!(PieceType::Pawn, Square::D2, Square::D3), // 16.
            mv!(PieceType::Pawn, Square::F5, Square::F4),
            mv!(PieceType::Queen, Square::G3, Square::F2), // 17.
            mv!(PieceType::Pawn, Square::E4, Square::E3),
            mv!(PieceType::Queen, Square::F2, Square::F3), // 18.
            mv!(PieceType::Queen, Square::D8, Square::H4),
            mv!(PieceType::Queen, Square::F3, Square::D5), // 19.
            mv!(PieceType::King, Square::G8, Square::H8),
            mv!(PieceType::Knight, Square::E1, Square::F3), // 20.
            mv!(PieceType::Queen, Square::H4, Square::F2),
            mv!(PieceType::King, Square::G1, Square::H1), // 21.
            mv!(PieceType::Knight, Square::G6, Square::H4),
            mv!(PieceType::Queen, Square::D5, Square::G5), // 22.
            mv!(PieceType::Bishop, Square::C8, Square::H3),
        ];
        for one in moves.iter() {
            game.make_move(Action::MakeMove(*one)).unwrap();
        }

        let direct_calculated_hash = ZOBRIST.calculate_position_hash(&game.get_position());
        let live_updating_hash = game.get_position().get_hash();
        assert_eq!(direct_calculated_hash, live_updating_hash);
    }
}
