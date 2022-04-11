use super::MoveTable;
use crate::bitboards::{BitBoard, BLANK};
use crate::square::{Square, SQUARES_NUMBER};

pub fn generate_pawn_moves(pawn_moves: &mut MoveTable) {
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
            if (diffs.0 == -1) & (diffs.1 == 0) | (diffs.0 == -2) & (diffs.1 == 0) & (rank == 1) {
                destination_mask = destination_mask | BitBoard::from_square(s_);
            }
        }
        pawn_moves.set_moves(source_square, destination_mask);
    }
}

pub fn generate_pawn_captures(pawn_captures: &mut MoveTable) {
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
            if (diffs.0 == -1) & (diffs.1.abs() == 1) {
                destination_mask = destination_mask | BitBoard::from_square(s_);
            }
        }
        pawn_captures.set_captures(source_square, destination_mask);
    }
}

#[rustfmt::skip]
#[cfg(test)]
mod tests {
    use super::*;
    use unindent::unindent;

    #[test]
    fn create() {
        let mut move_table = MoveTable::new();
        generate_pawn_moves(&mut move_table);
        generate_pawn_captures(&mut move_table);
        let square = Square::E4;
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
        println!("{}", move_table.get_moves(square));
        assert_eq!(
            format!("{}", move_table.get_moves(square)), unindent(result_str)
        );
    }

    #[test]
    fn create_2nd_rank() {
        let mut move_table = MoveTable::new();
        generate_pawn_moves(&mut move_table);
        generate_pawn_captures(&mut move_table);        let square = Square::E2;
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
        println!("{}", move_table.get_moves(square));
        assert_eq!(
            format!("{}", move_table.get_moves(square)), unindent(result_str)
        );
    }

    #[test]
    fn captures() {
        let mut move_table = MoveTable::new();
        generate_pawn_moves(&mut move_table);
        generate_pawn_captures(&mut move_table);
        let square = Square::E3;
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
        println!("{}", move_table.get_captures(square));
        assert_eq!(
            format!("{}", move_table.get_captures(square)), unindent(result_str)
        );
    }
}