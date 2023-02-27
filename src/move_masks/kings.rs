use super::PieceMoveTable;
use crate::{BitBoard, Square, BLANK, SQUARES_NUMBER};

fn get_rank_file_idx(square_idx: u8) -> (i32, i32) {
    let square = Square::new(square_idx).unwrap();
    (
        square.get_rank().to_index() as i32,
        square.get_file().to_index() as i32,
    )
}

pub fn generate_king_moves(table: &mut PieceMoveTable) {
    for source_index in 0..SQUARES_NUMBER as u8 {
        let source_square = Square::new(source_index).unwrap();
        let (rank, file) = get_rank_file_idx(source_index);
        let mut destination_mask = BLANK;
        for destination_index in 0..SQUARES_NUMBER as u8 {
            let (rank_, file_) = get_rank_file_idx(destination_index);
            let distances = ((rank - rank_).abs(), (file - file_).abs());
            if (distances.0 <= 1) & (distances.1 <= 1) {
                destination_mask |= BitBoard::from_square(Square::new(destination_index).unwrap());
            }
        }
        destination_mask ^= BitBoard::from_square(source_square);
        table.set_moves(source_square, destination_mask);
    }
}

#[rustfmt::skip]
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
