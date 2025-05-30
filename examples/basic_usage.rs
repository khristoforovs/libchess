use libchess::{ChessBoard, BoardMove, squares::*, PieceType::*, PieceMove, mv};

fn main() {
    // Create starting position
    let mut board = ChessBoard::default();
    
    // Generate legal moves
    let moves = board.get_legal_moves();
    println!("Available moves: {}", moves.len());
    
    // Make e2-e4 move
    let e2e4 = mv!(Queen, A1, A8);

    board.make_move(&e2e4);
    println!("Position after e4:\n{}", board);
}
