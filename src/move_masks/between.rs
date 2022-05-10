use crate::boards::{BitBoard, BLANK};
use crate::boards::{File, Rank};
use crate::boards::{Square, SQUARES_NUMBER};
use std::cmp::max;

pub struct BetweenTable([[Option<BitBoard>; SQUARES_NUMBER]; SQUARES_NUMBER]);

impl BetweenTable {
    pub fn new() -> Self {
        Self([[None; SQUARES_NUMBER]; SQUARES_NUMBER])
    }

    pub fn set(&mut self, square_a: Square, square_b: Square, value: Option<BitBoard>) {
        let (mut ai, mut bi) = (square_a.to_index(), square_b.to_index());
        if ai > bi {
            (ai, bi) = (bi, ai);
        }
        self.0[ai][bi] = value;
    }

    pub fn get(&self, square_a: Square, square_b: Square) -> Option<BitBoard> {
        let (mut ai, mut bi) = (square_a.to_index(), square_b.to_index());
        if ai > bi {
            (ai, bi) = (bi, ai);
        }
        self.0[ai][bi]
    }
}

pub fn generate_between_masks(table: &mut BetweenTable) {
    for index_a in 0..SQUARES_NUMBER {
        for index_b in index_a..SQUARES_NUMBER {
            let square_a = Square::new(index_a as u8).unwrap();
            let (rank_a, file_a) = (
                square_a.get_rank().to_index() as i32,
                square_a.get_file().to_index() as i32,
            );

            let square_b = Square::new(index_b as u8).unwrap();
            let (rank_b, file_b) = (
                square_b.get_rank().to_index() as i32,
                square_b.get_file().to_index() as i32,
            );

            if square_a == square_b {
                table.set(square_a, square_b, Some(BLANK));
            } else {
                let dist = ((rank_a - rank_b).abs(), (file_a - file_b).abs());
                if (dist.0 == dist.1) | (dist.0 == 0) | (dist.1 == 0) {
                    let mut mask = BLANK;
                    let max_distance = max(dist.0, dist.1);
                    for i in 1..max_distance {
                        mask |= BitBoard::from_rank_file(
                            Rank::from_index(
                                (rank_a + (rank_b - rank_a) / max_distance * i) as usize,
                            )
                            .unwrap(),
                            File::from_index(
                                (file_a + (file_b - file_a) / max_distance * i) as usize,
                            )
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

#[rustfmt::skip]
#[cfg(test)]
mod tests {
    use super::*;
    use unindent::unindent;

    #[test]
    fn between_diagonal() {
        let mut between_table = BetweenTable::new();
        generate_between_masks(&mut between_table);
        let (square_a, square_b) = (Square::C3, Square::G7);
        let result_str = 
            ". . . . . . . . 
             . . . . . . . . 
             . . . . . X . . 
             . . . . X . . . 
             . . . X . . . . 
             . . . . . . . . 
             . . . . . . . . 
             . . . . . . . . 
            ";
        println!("{}", between_table.get(square_a, square_b).unwrap());
        assert_eq!(
            format!("{}", between_table.get(square_a, square_b).unwrap()),
            unindent(result_str)
        );
    }

    #[test]
    fn between_vertical() {
        let mut between_table = BetweenTable::new();
        generate_between_masks(&mut between_table);
        let (square_a, square_b) = (Square::D5, Square::D1);
        let result_str = 
            ". . . . . . . . 
             . . . . . . . . 
             . . . . . . . . 
             . . . . . . . . 
             . . . X . . . . 
             . . . X . . . . 
             . . . X . . . . 
             . . . . . . . . 
            ";
        println!("{}", between_table.get(square_a, square_b).unwrap());
        assert_eq!(
            format!("{}", between_table.get(square_a, square_b).unwrap()),
            unindent(result_str)
        );
    }

    #[test]
    fn between_point() {
        let mut between_table = BetweenTable::new();
        generate_between_masks(&mut between_table);
        let (square_a, square_b) = (Square::D5, Square::D5);
        let result_str = 
            ". . . . . . . . 
             . . . . . . . . 
             . . . . . . . . 
             . . . . . . . . 
             . . . . . . . . 
             . . . . . . . . 
             . . . . . . . . 
             . . . . . . . . 
            ";
        println!("{}", between_table.get(square_a, square_b).unwrap());
        assert_eq!(
            format!("{}", between_table.get(square_a, square_b).unwrap()),
            unindent(result_str)
        );
    }

    #[test]
    fn between_empty() {
        let mut between_table = BetweenTable::new();
        generate_between_masks(&mut between_table);
        let (square_a, square_b) = (Square::D5, Square::C3);
        assert!(between_table.get(square_a, square_b).is_none());
    }
}
