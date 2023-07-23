use crate::{BitBoard, File, Rank, Square, BLANK, SQUARES_NUMBER};
use std::cmp::max;

const TABLE_SIZE: usize = SQUARES_NUMBER * (SQUARES_NUMBER + 1) / 2;

pub struct BetweenTable([Option<BitBoard>; TABLE_SIZE]);

impl Default for BetweenTable {
    fn default() -> Self { Self::new() }
}

impl BetweenTable {
    pub fn new() -> Self { Self([None; TABLE_SIZE]) }

    pub fn set(&mut self, square_a: Square, square_b: Square, value: Option<BitBoard>) {
        let (mut ai, mut bi) = (square_a.to_index(), square_b.to_index());
        if ai > bi {
            (ai, bi) = (bi, ai);
        }
        let ai_i = ai as i64;
        let offset = (SQUARES_NUMBER as i64 * ai_i - (ai_i - 1) * ai_i / 2) as usize;
        self.0[offset + bi - ai] = value;
    }

    pub fn get(&self, square_a: Square, square_b: Square) -> Option<BitBoard> {
        let (mut ai, mut bi) = (square_a.to_index(), square_b.to_index());
        if ai > bi {
            (ai, bi) = (bi, ai);
        }
        let ai_i = ai as i64;
        let offset = (SQUARES_NUMBER as i64 * ai_i - (ai_i - 1) * ai_i / 2) as usize;
        self.0[offset + bi - ai]
    }
}

pub fn generate_between_masks(table: &mut BetweenTable) {
    for index_a in 0..SQUARES_NUMBER as u8 {
        let square_a = Square::new(index_a).unwrap();
        let (rank_a, file_a) = (
            square_a.get_rank().to_index() as i32,
            square_a.get_file().to_index() as i32,
        );

        for index_b in index_a..SQUARES_NUMBER as u8 {
            let square_b = Square::new(index_b).unwrap();
            if square_a == square_b {
                table.set(square_a, square_b, Some(BLANK));
            } else {
                let diff = square_a.offsets_from(square_b);
                let dist = (diff.0.abs(), diff.1.abs());

                if (dist.0 == dist.1) | (dist.0 == 0) | (dist.1 == 0) {
                    let mut mask = BLANK;
                    let max_distance = max(dist.0, dist.1);
                    for i in 1..max_distance {
                        mask |= BitBoard::from_rank_file(
                            Rank::from_index((rank_a + diff.0 / max_distance * i) as usize)
                                .unwrap(),
                            File::from_index((file_a + diff.1 / max_distance * i) as usize)
                                .unwrap(),
                        );
                    }
                    table.set(square_a, square_b, Some(mask));
                } else {
                    table.set(square_a, square_b, None);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::squares::*;

    #[test]
    fn between_diagonal() {
        let mut between_table = BetweenTable::new();
        generate_between_masks(&mut between_table);
        let (square_a, square_b) = (C3, G7);
        let table = between_table.get(square_a, square_b).unwrap();
        let result = 0x0000201008000000u64;
        println!("{}", table);
        assert_eq!(table.bits(), result);
    }

    #[test]
    fn between_vertical() {
        let mut between_table = BetweenTable::new();
        generate_between_masks(&mut between_table);
        let (square_a, square_b) = (D5, D1);
        let table = between_table.get(square_a, square_b).unwrap();
        let result = 0x0000000008080800u64;
        println!("{}", table);
        assert_eq!(table.bits(), result);
    }

    #[test]
    fn between_point() {
        let mut between_table = BetweenTable::new();
        generate_between_masks(&mut between_table);
        let (square_a, square_b) = (D5, D5);
        let table = between_table.get(square_a, square_b).unwrap();
        let result = 0u64;
        println!("{}", table);
        assert_eq!(table.bits(), result);
    }

    #[test]
    fn between_empty() {
        let mut between_table = BetweenTable::new();
        generate_between_masks(&mut between_table);
        let (square_a, square_b) = (D5, C3);
        assert!(between_table.get(square_a, square_b).is_none());
    }
}
