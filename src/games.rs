use crate::bitboards::BLANK;
use crate::chess_boards::ChessBoard;
use crate::chess_moves::ChessMove;
use crate::colors::Color;
use crate::errors::{ChessBoardError, GameError};
use crate::pieces::PieceType;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

const HISTORY_CAPACITY: usize = 100;
const UNIQUE_POSITIONS_CAPACITY: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    MakeMove(ChessMove),
    OfferDraw,
    AcceptDraw,
    DeclineDraw,
    Resign,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    Ongoing,
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
            GameStatus::Ongoing => String::from("the game is ongoing"),
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

#[derive(Debug, Clone)]
pub struct Game {
    position: ChessBoard,
    history: Vec<Action>,
    unique_positions_counter: HashMap<u64, usize>,
    status: GameStatus,
}

impl Default for Game {
    #[inline]
    fn default() -> Self {
        let mut result = Self {
            position: ChessBoard::default(),
            history: Vec::with_capacity(HISTORY_CAPACITY),
            unique_positions_counter: HashMap::with_capacity(UNIQUE_POSITIONS_CAPACITY),
            status: GameStatus::Ongoing,
        };

        result.update_game_status().position_counter_increment();
        result
    }
}

impl Game {
    #[inline]
    pub fn from_board(board: ChessBoard) -> Self {
        let mut result = Self {
            position: board,
            history: Vec::with_capacity(HISTORY_CAPACITY),
            unique_positions_counter: HashMap::with_capacity(UNIQUE_POSITIONS_CAPACITY),
            status: GameStatus::Ongoing,
        };

        result.update_game_status().position_counter_increment();
        result
    }

    #[inline]
    pub fn from_fen(fen: &str) -> Result<Self, ChessBoardError> {
        match ChessBoard::from_str(&fen) {
            Ok(board) => Ok(Self::from_board(board)),
            Err(e) => Err(e),
        }
    }

    #[inline]
    pub fn get_action_history(&self) -> &Vec<Action> {
        &self.history
    }

    #[inline]
    pub fn get_position(&self) -> &ChessBoard {
        &self.position
    }

    #[inline]
    pub fn get_game_status(&self) -> GameStatus {
        self.status
    }

    #[inline]
    pub fn get_side_to_move(&self) -> Color {
        self.get_position().get_side_to_move()
    }

    #[inline]
    pub fn get_last_action(&self) -> Option<Action> {
        self.get_action_history().last().cloned()
    }

    #[inline]
    pub fn get_position_counter(&self, position: &ChessBoard) -> usize {
        match self.unique_positions_counter.get(&position.get_hash()) {
            Some(counter) => *counter,
            None => 0,
        }
    }

    #[inline]
    fn set_game_status(&mut self, status: GameStatus) -> &mut Self {
        self.status = status;
        self
    }

    fn update_game_status(&mut self) -> &mut Self {
        self.set_game_status(match self.get_last_action() {
            None | Some(Action::MakeMove(_)) => {
                let position = self.get_position();
                if position.get_legal_moves().len() == 0 {
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
            Some(Action::OfferDraw) => GameStatus::Ongoing,
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

    #[inline]
    fn push_action_to_history(&mut self, action: Action) -> &mut Self {
        self.history.push(action);
        self
    }

    pub fn make_move(&mut self, action: Action) -> Result<&mut Self, GameError> {
        if self.get_game_status() != GameStatus::Ongoing {
            return Err(GameError::GameIsAlreadyFinished);
        }

        match action {
            Action::MakeMove(next_move) => match self.get_position().make_move(next_move) {
                Ok(next_position) => {
                    self.set_position(next_position);
                    self.position_counter_increment();
                }
                Err(_) => {
                    return Err(GameError::IllegalActionDetected);
                }
            },
            Action::DeclineDraw | Action::AcceptDraw => {
                if let Some(last) = self.get_last_action() {
                    if last != Action::OfferDraw {
                        return Err(GameError::DrawOfferNeedsAnswer);
                    }
                } else {
                    return Err(GameError::IllegalActionDetected);
                }
            }
            Action::OfferDraw | Action::Resign => {}
        };

        self.push_action_to_history(action);
        self.update_game_status();
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::{pieces::PieceType, squares::Square};

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
}
