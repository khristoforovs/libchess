use libchess::*;
use libchess::{PieceType::*, squares::*};
use std::str::FromStr;

fn main() {
    let mut game = Game::default();
    let moves = vec![
        "e2e4", "c7c5",
        "Ng1f3", "d7d6",
        "d2d4", "c5d4",
        "Nf3d4", "Ng8f6",
    ];

    moves.into_iter().for_each(|one| {
        game.make_move(&Action::MakeMove(mv_str!(one))).unwrap();
    });
    println!("{}", game.get_position());
}
