use libchess::*;
use libchess::{PieceType::*, squares::*, Color::*};

fn main() {
    let mut game = Game::default();
    let moves = vec![
        mv!(Pawn, E2, E4),
        mv!(Pawn, E7, E5),
        mv!(Queen, D1, H5),
        mv!(King, E8, E7),
        mv!(Queen, H5, E5),
    ];

    moves.into_iter().for_each(|one| {
        game.make_move(&Action::MakeMove(one)).unwrap();
    });

    println!("{}", game.get_position());
    assert_eq!(game.get_game_status(), GameStatus::CheckMated(Black));
}
