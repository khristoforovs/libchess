//! The chess board module
//!
//! The module implements the board itself, board's coordinates
//! (files and ranks), the low-level part of board calculations -
//! bitboard and the builder-module which prepares the board's
//! initialization data

mod bitboards;
pub use bitboards::{BitBoard, BLANK};

mod board_builders;
pub use board_builders::BoardBuilder;

mod board_files;
pub use board_files::{File, FILES, FILES_NUMBER};

mod board_ranks;
pub use board_ranks::{Rank, RANKS, RANKS_NUMBER};

mod coordinates;
pub use coordinates::{squares, Square, SQUARES_NUMBER};

mod chess_boards;
pub use chess_boards::{BoardStatus, ChessBoard, LegalMoves};

mod zobrist;
pub use zobrist::{PositionHashValueType, ZOBRIST_TABLES};

#[macro_use]
mod board_moves;
pub use board_moves::{
    BoardMove, BoardMoveOption, DisplayAmbiguityType, PieceMove, PromotionPieceType,
};
