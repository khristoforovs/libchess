use super::PieceMoveTable;
use crate::{BitBoard, Square, BLANK, SQUARES_NUMBER};

pub fn generate_king_moves(table: &mut PieceMoveTable) {
    for source_index in 0..SQUARES_NUMBER as u8 {
        let source_square = Square::new(source_index).unwrap();

        let mut destination_mask = BLANK;
        for destination_index in 0..SQUARES_NUMBER as u8 {
            let destination_square = Square::new(destination_index).unwrap();
            let diffs = source_square.offsets_from(destination_square);
            let distances = (diffs.0.abs(), diffs.1.abs());

            if (distances.0 <= 1) & (distances.1 <= 1) {
                destination_mask |= BitBoard::from_square(Square::new(destination_index).unwrap());
            }
        }
        destination_mask ^= BitBoard::from_square(source_square);
        table.set_moves(source_square, destination_mask);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::squares::*;

    #[test]
    fn create() {
        let mut move_table = PieceMoveTable::new();
        generate_king_moves(&mut move_table);
        let square = E4;
        let result = 0x000000003828380000u64;
        let table = move_table.get_moves(square);
        println!("{}", table);
        assert_eq!(table.bits(), result);
    }
}
