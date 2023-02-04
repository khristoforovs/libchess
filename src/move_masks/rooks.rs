use super::{PieceMoveTable, RAYS_TABLE};
use crate::{Square, BLANK, SQUARES_NUMBER};

pub fn generate_rook_moves(table: &mut PieceMoveTable) {
    for source_index in 0..SQUARES_NUMBER {
        let source_square = Square::new(source_index as u8).unwrap();
        let mut destination_mask = BLANK;
        let rays = RAYS_TABLE.get(source_square);
        for i in 0..4 {
            destination_mask |= rays[i];
        }
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
        generate_rook_moves(&mut move_table);
        let square = E4;
        let result = 0x10101010ef101010u64;
        let table = move_table.get_moves(square);
        println!("{}", table);
        assert_eq!(table.0, result);
    }
}
