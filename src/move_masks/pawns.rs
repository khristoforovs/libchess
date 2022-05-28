use crate::boards::{BitBoard, BLANK};
use crate::boards::{Square, SQUARES_NUMBER};
use crate::{Color, COLORS_NUMBER};

pub struct PawnMoveTable {
    moves: [BitBoard; SQUARES_NUMBER * COLORS_NUMBER],
    captures: [BitBoard; SQUARES_NUMBER * COLORS_NUMBER],
}

impl Default for PawnMoveTable {
    fn default() -> Self {
        Self::new()
    }
}

impl PawnMoveTable {
    pub fn new() -> Self {
        Self {
            moves: [BLANK; SQUARES_NUMBER * COLORS_NUMBER],
            captures: [BLANK; SQUARES_NUMBER * COLORS_NUMBER],
        }
    }

    pub fn set_moves(&mut self, square: Square, color: Color, value: BitBoard) {
        let index = square.to_index() + SQUARES_NUMBER * color.to_index();
        self.moves[index] = value;
    }

    pub fn set_captures(&mut self, square: Square, color: Color, value: BitBoard) {
        let index = square.to_index() + SQUARES_NUMBER * color.to_index();
        self.captures[index] = value;
    }

    pub fn reset_moves(&mut self) {
        self.moves = [BLANK; SQUARES_NUMBER * COLORS_NUMBER];
    }

    pub fn reset_captures(&mut self) {
        self.captures = [BLANK; SQUARES_NUMBER * COLORS_NUMBER]
    }

    pub fn get_moves(&self, square: Square, color: Color) -> BitBoard {
        let index = square.to_index() + SQUARES_NUMBER * color.to_index();
        self.moves[index]
    }

    pub fn get_captures(&self, square: Square, color: Color) -> BitBoard {
        let index = square.to_index() + SQUARES_NUMBER * color.to_index();
        self.captures[index]
    }
}

pub fn generate_pawn_moves(table: &mut PawnMoveTable, color: Color) {
    for source_index in 0..SQUARES_NUMBER {
        let source_square = Square::new(source_index as u8).unwrap();
        let (rank, file) = (
            source_square.get_rank().to_index() as i32,
            source_square.get_file().to_index() as i32,
        );

        let mut destination_mask = BLANK;
        for destination_index in 0..SQUARES_NUMBER {
            let s_ = Square::new(destination_index as u8).unwrap();
            let diffs = (
                (rank - s_.get_rank().to_index() as i32),
                (file - s_.get_file().to_index() as i32),
            );
            match color {
                Color::White => {
                    if (diffs.0 == -1) & (diffs.1 == 0)
                        | (diffs.0 == -2) & (diffs.1 == 0) & (rank == 1)
                    {
                        destination_mask |= BitBoard::from_square(s_);
                    }
                }
                Color::Black => {
                    if (diffs.0 == 1) & (diffs.1 == 0)
                        | (diffs.0 == 2) & (diffs.1 == 0) & (rank == 6)
                    {
                        destination_mask |= BitBoard::from_square(s_);
                    }
                }
            }
        }
        table.set_moves(source_square, color, destination_mask);
    }
}

pub fn generate_pawn_captures(table: &mut PawnMoveTable, color: Color) {
    for source_index in 0..SQUARES_NUMBER {
        let source_square = Square::new(source_index as u8).unwrap();
        let (rank, file) = (
            source_square.get_rank().to_index() as i32,
            source_square.get_file().to_index() as i32,
        );

        let mut destination_mask = BLANK;
        for destination_index in 0..SQUARES_NUMBER {
            let s_ = Square::new(destination_index as u8).unwrap();
            let diffs = (
                (rank - s_.get_rank().to_index() as i32),
                (file - s_.get_file().to_index() as i32),
            );
            match color {
                Color::White => {
                    if (diffs.0 == -1) & (diffs.1.abs() == 1) {
                        destination_mask |= BitBoard::from_square(s_);
                    }
                }
                Color::Black => {
                    if (diffs.0 == 1) & (diffs.1.abs() == 1) {
                        destination_mask |= BitBoard::from_square(s_);
                    }
                }
            }
        }
        table.set_captures(source_square, color, destination_mask);
    }
}

#[rustfmt::skip]
#[cfg(test)]
mod tests {
    use super::*;
    use unindent::unindent;
    use crate::boards::squares::*;

    #[test]
    fn create() {
        let mut move_table = PawnMoveTable::new();
        generate_pawn_moves(&mut move_table, Color::White);
        generate_pawn_moves(&mut move_table, Color::Black);
        let square = E4;
        let result_str = 
            ". . . . . . . . 
             . . . . . . . . 
             . . . . . . . . 
             . . . . X . . . 
             . . . . . . . . 
             . . . . . . . . 
             . . . . . . . . 
             . . . . . . . . 
            ";
        println!("{}", move_table.get_moves(square, Color::White));
        assert_eq!(
            format!("{}", move_table.get_moves(square, Color::White)), unindent(result_str)
        );

        let square = E5;
        let result_str = 
            ". . . . . . . . 
             . . . . . . . . 
             . . . . . . . . 
             . . . . . . . . 
             . . . . X . . . 
             . . . . . . . . 
             . . . . . . . . 
             . . . . . . . . 
            ";
        println!("{}", move_table.get_moves(square, Color::Black));
        assert_eq!(
            format!("{}", move_table.get_moves(square, Color::Black)), unindent(result_str)
        );
    }

    #[test]
    fn create_2nd_7th_rank() {
        let mut move_table = PawnMoveTable::new();
        generate_pawn_moves(&mut move_table, Color::White);
        generate_pawn_moves(&mut move_table, Color::Black);
        let square = E2;
        let result_str = 
            ". . . . . . . . 
             . . . . . . . . 
             . . . . . . . . 
             . . . . . . . . 
             . . . . X . . . 
             . . . . X . . . 
             . . . . . . . . 
             . . . . . . . . 
            ";
        println!("{}", move_table.get_moves(square, Color::White));
        assert_eq!(
            format!("{}", move_table.get_moves(square, Color::White)), unindent(result_str)
        );

        let square = E7;
        let result_str = 
            ". . . . . . . . 
             . . . . . . . . 
             . . . . X . . . 
             . . . . X . . . 
             . . . . . . . . 
             . . . . . . . . 
             . . . . . . . . 
             . . . . . . . . 
            ";
        println!("{}", move_table.get_moves(square, Color::Black));
        assert_eq!(
            format!("{}", move_table.get_moves(square, Color::Black)), unindent(result_str)
        );
    }

    #[test]
    fn captures() {
        let mut move_table = PawnMoveTable::new();
        generate_pawn_captures(&mut move_table, Color::White);
        generate_pawn_captures(&mut move_table, Color::Black);
        let square = E3;
        let result_str = 
            ". . . . . . . . 
             . . . . . . . . 
             . . . . . . . . 
             . . . . . . . . 
             . . . X . X . . 
             . . . . . . . . 
             . . . . . . . . 
             . . . . . . . . 
            ";
        println!("{}", move_table.get_captures(square, Color::White));
        assert_eq!(
            format!("{}", move_table.get_captures(square, Color::White)), unindent(result_str)
        );

        let square = E6;
        let result_str = 
            ". . . . . . . . 
             . . . . . . . . 
             . . . . . . . . 
             . . . X . X . . 
             . . . . . . . . 
             . . . . . . . . 
             . . . . . . . . 
             . . . . . . . . 
            ";
        println!("{}", move_table.get_captures(square, Color::Black));
        assert_eq!(
            format!("{}", move_table.get_captures(square, Color::Black)), unindent(result_str)
        );
    }
}
