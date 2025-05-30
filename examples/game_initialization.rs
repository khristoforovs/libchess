use libchess::*;
use libchess::{PieceType::*, squares::*};

fn main() {
    // initializing the game is almost the same as for boards
    let mut game = Game::from_fen("3k4/3P4/4K3/8/8/8/8/8 w - - 0 1").unwrap();
    let moves = vec![mv!(King, E6, D6)]; // defining vec of chess moves to be applied to the board
    moves.into_iter().for_each(|one| {
        game.make_move(&Action::MakeMove(one)).unwrap();
    });

    println!("{}", game.get_position());
    assert_eq!(game.get_game_status(), GameStatus::Stalemate);
}
