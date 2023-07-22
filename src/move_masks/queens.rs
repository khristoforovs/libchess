use super::{PieceMoveTable, RAYS_TABLE};
use crate::{Square, BLANK, SQUARES_NUMBER};

pub fn generate_queen_moves(table: &mut PieceMoveTable) {
    for source_index in 0..SQUARES_NUMBER as u8 {
        let source_square = Square::new(source_index).unwrap();
        let mut destination_mask = BLANK;
        let rays = RAYS_TABLE.get(source_square);
        (0..8).for_each(|i| destination_mask |= rays[i]);
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
        generate_queen_moves(&mut move_table);
        let square = E4;
        let result = 0x11925438ef385492u64;
        let table = move_table.get_moves(square);
        println!("{}", table);
        assert_eq!(table.bits(), result);
    }
}
