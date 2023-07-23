use crate::{BitBoard, Square, BLANK, SQUARES_NUMBER};

/// This masks structure contains all available rays for each square on the board.
/// For the most generic case, we need to store 8 different directions: both side
/// vertical + both side horizontal and 4 diagonal rays. This masks are useful for
/// faster generation of generic move tables for long-range pieces and also allows
/// us to calculate faster possible moves when paths are blocked by other pieces.
///
/// Indexing of rays:
/// 0: Up, 1: Down, 2: Right, 3: Left,
/// 4: Up-Right, 5: Up-Left, 6: Down-Right, 7: Down-Left
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

    for source_index in 0..SQUARES_NUMBER as u8 {
        let source_square = Square::new(source_index).unwrap();

        let mut mask = [BLANK; 8];
        for destination_index in 0..SQUARES_NUMBER as u8 {
            let destination_square = Square::new(destination_index).unwrap();
            let diffs = source_square.offsets_from(destination_square);

            conditions
                .iter()
                .enumerate()
                .filter(|val| val.1(diffs.0, diffs.1))
                .for_each(|val| {
                    mask[val.0] |= BitBoard::from_square(destination_square);
                });
        }
        table.set(source_square, mask);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::squares::*;

    #[test]
    fn create() {
        let rays_table = RaysTable::default();
        let square = E1;
        assert_eq!(
            rays_table.get(square)[0],
            BitBoard::new(0x1010101010101000u64)
        );
        assert_eq!(
            rays_table.get(square)[4],
            BitBoard::new(0x0000000080402000u64)
        );

        let square = D4;
        assert_eq!(
            rays_table.get(square)[1],
            BitBoard::new(0x0000000000080808u64)
        );
        assert_eq!(
            rays_table.get(square)[5],
            BitBoard::new(0x0001020400000000u64)
        );

        let square = G6;
        assert_eq!(
            rays_table.get(square)[2],
            BitBoard::new(0x0000800000000000u64)
        );
        assert_eq!(
            rays_table.get(square)[6],
            BitBoard::new(0x0000008000000000u64)
        );
    }
}
