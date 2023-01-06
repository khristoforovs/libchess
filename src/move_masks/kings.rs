use super::PieceMoveTable;
use crate::boards::{BitBoard, BLANK};
use crate::boards::{Square, SQUARES_NUMBER};

pub fn generate_king_moves(table: &mut PieceMoveTable) {
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
            if (distances.0 <= 1) & (distances.1 <= 1) {
                destination_mask |= BitBoard::from_square(s_);
            }
        }
        let source_mask = BitBoard::from_square(source_square);
        destination_mask ^= source_mask;
        table.set_moves(source_square, destination_mask);
    }
}

#[rustfmt::skip]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::boards::squares::*;

    #[test]
    fn create() {
        let mut move_table = PieceMoveTable::new();
        generate_king_moves(&mut move_table);
        let square = E4;
        let result = 0x000000003828380000u64;
        let table = move_table.get_moves(square);
        println!("{}", table);
        assert_eq!(table.0, result);
    }
}
