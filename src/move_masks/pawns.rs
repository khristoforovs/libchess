use crate::{BitBoard, Color, Square, BLANK, COLORS_NUMBER, SQUARES_NUMBER};

pub struct PawnMoveTable {
    moves:    [BitBoard; SQUARES_NUMBER * COLORS_NUMBER],
    captures: [BitBoard; SQUARES_NUMBER * COLORS_NUMBER],
}

impl Default for PawnMoveTable {
    fn default() -> Self { Self::new() }
}

impl PawnMoveTable {
    pub fn new() -> Self {
        Self {
            moves:    [BLANK; SQUARES_NUMBER * COLORS_NUMBER],
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

    pub fn reset_moves(&mut self) { self.moves = [BLANK; SQUARES_NUMBER * COLORS_NUMBER]; }

    pub fn reset_captures(&mut self) { self.captures = [BLANK; SQUARES_NUMBER * COLORS_NUMBER] }

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
    for source_index in 0..SQUARES_NUMBER as u8 {
        let source_square = Square::new(source_index).unwrap();
        let source_rank = source_square.get_rank().to_index();

        let mut dest_mask = BLANK;
        (0..SQUARES_NUMBER as u8).for_each(|dest_index| {
            let destination_square = Square::new(dest_index).unwrap();
            let d = source_square.offsets_from(destination_square);

            match color {
                Color::White => {
                    if (d.0 == 1) & (d.1 == 0) | (d.0 == 2) & (d.1 == 0) & (source_rank == 1) {
                        dest_mask |= BitBoard::from_square(Square::new(dest_index).unwrap());
                    }
                }
                Color::Black => {
                    if (d.0 == -1) & (d.1 == 0) | (d.0 == -2) & (d.1 == 0) & (source_rank == 6) {
                        dest_mask |= BitBoard::from_square(Square::new(dest_index).unwrap());
                    }
                }
            }
        });
        table.set_moves(source_square, color, dest_mask);
    }
}

pub fn generate_pawn_captures(table: &mut PawnMoveTable, color: Color) {
    for source_index in 0..SQUARES_NUMBER as u8 {
        let source_square = Square::new(source_index).unwrap();

        let mut dest_mask = BLANK;
        (0..SQUARES_NUMBER as u8).for_each(|dest_index| {
            let destination_square = Square::new(dest_index).unwrap();
            let d = source_square.offsets_from(destination_square);

            match color {
                Color::White => {
                    if (d.0 == 1) & (d.1.abs() == 1) {
                        dest_mask |= BitBoard::from_square(Square::new(dest_index).unwrap());
                    }
                }
                Color::Black => {
                    if (d.0 == -1) & (d.1.abs() == 1) {
                        dest_mask |= BitBoard::from_square(Square::new(dest_index).unwrap());
                    }
                }
            }
        });
        table.set_captures(source_square, color, dest_mask);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::squares::*;

    #[test]
    fn create() {
        let mut move_table = PawnMoveTable::new();
        generate_pawn_moves(&mut move_table, Color::White);
        generate_pawn_moves(&mut move_table, Color::Black);

        let square = E4;
        let result = 0x0000001000000000u64;
        let table = move_table.get_moves(square, Color::White);
        println!("{}", table);
        assert_eq!(table.bits(), result);

        let square = E5;
        let result = 0x0000000010000000u64;
        let table = move_table.get_moves(square, Color::Black);
        println!("{}", table);
        assert_eq!(table.bits(), result);
    }

    #[test]
    fn create_2nd_7th_rank() {
        let mut move_table = PawnMoveTable::new();
        generate_pawn_moves(&mut move_table, Color::White);
        generate_pawn_moves(&mut move_table, Color::Black);

        let square = E2;
        let result = 0x0000000010100000u64;
        let table = move_table.get_moves(square, Color::White);
        println!("{}", table);
        assert_eq!(table.bits(), result);

        let square = E7;
        let result = 0x0000101000000000u64;
        let table = move_table.get_moves(square, Color::Black);
        println!("{}", table);
        assert_eq!(table.bits(), result);
    }

    #[test]
    fn captures() {
        let mut move_table = PawnMoveTable::new();
        generate_pawn_captures(&mut move_table, Color::White);
        generate_pawn_captures(&mut move_table, Color::Black);

        let square = E3;
        let result = 0x0000000028000000u64;
        let table = move_table.get_captures(square, Color::White);
        println!("{}", table);
        assert_eq!(table.bits(), result);

        let square = E6;
        let result = 0x0000002800000000u64;
        let table = move_table.get_captures(square, Color::Black);
        println!("{}", table);
        assert_eq!(table.bits(), result);
    }
}
