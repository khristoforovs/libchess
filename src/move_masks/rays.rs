use crate::{BitBoard, Square, BLANK, SQUARES_NUMBER};

pub struct RaysTable {
    rays: [[BitBoard; 8]; SQUARES_NUMBER],
}

impl Default for RaysTable {
    fn default() -> Self {
        let mut result = Self::new();
        generate_rays(&mut result);
        result
    }
}

impl RaysTable {
    fn new() -> Self {
        Self {
            rays: [[BLANK; 8]; SQUARES_NUMBER],
        }
    }

    pub fn set(&mut self, square: Square, value: [BitBoard; 8]) {
        self.rays[square.to_index()] = value;
    }

    pub fn reset(&mut self) { self.rays = [[BLANK; 8]; SQUARES_NUMBER]; }

    pub fn get(&self, square: Square) -> [BitBoard; 8] { self.rays[square.to_index()] }
}

fn generate_rays(table: &mut RaysTable) {
    let conditions = [
        |dy: i32, dx: i32| -> bool { (dy > 0) & (dx == 0) }, // up
        |dy: i32, dx: i32| -> bool { (dy < 0) & (dx == 0) }, // down
        |dy: i32, dx: i32| -> bool { (dy == 0) & (dx > 0) }, // right
        |dy: i32, dx: i32| -> bool { (dy == 0) & (dx < 0) }, // left
        |dy: i32, dx: i32| -> bool { (dy.abs() - dx.abs() == 0) & (dx > 0) & (dy > 0) }, // up-right
        |dy: i32, dx: i32| -> bool { (dy.abs() - dx.abs() == 0) & (dx < 0) & (dy > 0) }, // up-left
        |dy: i32, dx: i32| -> bool { (dy.abs() - dx.abs() == 0) & (dx > 0) & (dy < 0) }, /* down-right */
        |dy: i32, dx: i32| -> bool { (dy.abs() - dx.abs() == 0) & (dx < 0) & (dy < 0) }, /* down-left */
    ];

    for source_index in 0..SQUARES_NUMBER {
        let source_square = Square::new(source_index as u8).unwrap();
        let (rank, file) = (
            source_square.get_rank().to_index() as i32,
            source_square.get_file().to_index() as i32,
        );

        let mut destination_mask = [BLANK; 8];
        for destination_index in 0..SQUARES_NUMBER {
            let s_ = Square::new(destination_index as u8).unwrap();
            let diffs = (
                (s_.get_rank().to_index() as i32 - rank),
                (s_.get_file().to_index() as i32 - file),
            );

            for (i, condition) in conditions.iter().enumerate() {
                if condition(diffs.0, diffs.1) {
                    destination_mask[i] |= BitBoard::from_square(s_);
                }
            }
        }
        table.set(source_square, destination_mask);
    }
}

#[rustfmt::skip]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::squares::*;

    #[test]
    fn create() {
        let rays_table = RaysTable::default();
        let square = E1;
        assert_eq!(rays_table.get(square)[0], BitBoard::new(0x1010101010101000u64));
        assert_eq!(rays_table.get(square)[4], BitBoard::new(0x0000000080402000u64));

        let square = D4;
        assert_eq!(rays_table.get(square)[1], BitBoard::new(0x0000000000080808u64));
        assert_eq!(rays_table.get(square)[5], BitBoard::new(0x0001020400000000u64));

        let square = G6;
        assert_eq!(rays_table.get(square)[2], BitBoard::new(0x0000800000000000u64));
        assert_eq!(rays_table.get(square)[6], BitBoard::new(0x0000008000000000u64));
    }
}
