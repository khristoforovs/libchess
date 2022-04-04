use super::MoveTable;
use crate::bitboards::{BitBoard, BLANK};
use crate::square::{Square, SQUARES_NUMBER};

pub fn generate_queen_moves() -> MoveTable {
    let mut queen_moves: MoveTable = MoveTable::new();
    for source_index in 0..SQUARES_NUMBER {
        let source_square = Square::new(source_index as u8).unwrap();
        let (rank, file) = (
            source_square.get_rank().to_index() as i32,
            source_square.get_file().to_index() as i32,
        );

        let mut destination_mask = BLANK;
        for destination_index in 0..SQUARES_NUMBER {
            let s_ = Square::new(destination_index as u8).unwrap();
            let distances = (
                (rank - s_.get_rank().to_index() as i32).abs(),
                (file - s_.get_file().to_index() as i32).abs(),
            );
            if (distances.0 == distances.1) | (distances.0 == 0) | (distances.1 == 0) {
                destination_mask = destination_mask | BitBoard::from_square(s_);
            }
        }
        let source_mask = BitBoard::from_square(source_square);
        destination_mask ^= source_mask;
        queen_moves.insert(source_mask, destination_mask);
    }

    queen_moves
}

#[rustfmt::skip]
#[cfg(test)]
mod tests {
    use super::*;
    use unindent::unindent;

    #[test]
    fn create() {
        let move_table = generate_queen_moves();
        let bit_board = BitBoard::from_square(Square::E4);
        let result_str = 
            "X . . . X . . . 
             . X . . X . . X 
             . . X . X . X . 
             . . . X X X . . 
             X X X X . X X X 
             . . . X X X . . 
             . . X . X . X . 
             . X . . X . . X 
            ";
        println!("{}", move_table.get(&bit_board).unwrap());
        assert_eq!(
            format!("{}", move_table.get(&bit_board).unwrap()),
            unindent(result_str)
        );
    }
}
