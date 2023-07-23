use thiserror::Error;

#[derive(Error, Debug)]
pub enum LibChessError {
    #[error("Invalid index for board's file: {}", n)]
    InvalidBoardFileIndex { n: usize },

    #[error("Negative file index found")]
    NegativeBoardFileIndex,

    #[error("Invalid board's file string representation")]
    InvalidBoardFileName,

    #[error("Invalid index for board's rank: {}", n)]
    InvalidBoardRankIndex { n: usize },

    #[error("Negative rank index found")]
    NegativeBoardRankIndex,

    #[error("Invalid board's rank string representation")]
    InvalidBoardRankName,

    #[error("Invalid square representation string")]
    InvalidSquareRepresentation,

    #[error("Invalid castling index: only one from range 0..=3 is allowed")]
    InvalidCastlingIndexRepresentation,

    // Piece Errors
    #[error("Invalid peace representation string")]
    InvalidPeaceRepresentation,

    #[error("Invalid peace index : {}", n)]
    InvalidPeaceIndex { n: usize },

    #[error("Invalid color index : {}", n)]
    InvalidColorIndex { n: usize },

    // Board Moves Errors
    #[error("Invalid move representation string")]
    InvalidBoardMoveRepresentation,

    #[error("Pawn can't be promoted to pawn")]
    InvalidPromotionPiece,

    #[error("Invalid move for current board")]
    InvalidMoveForCurrentBoard,

    // Chess Board Errors
    #[error("Invalid FEN string: {}", s)]
    InvalidFENString { s: String },

    #[error("Invalid position: colors overlapping detected")]
    InvalidPositionColorsOverlap,

    #[error("Invalid position: 2 or more piece type overlap detected")]
    InvalidPositionPieceTypeOverlap,

    #[error("Invalid board: combined mask is not self-consistent")]
    InvalidBoardSelfNonConsistency,

    #[error("Invalid board: more than 1 king of the same color")]
    InvalidBoardMultipleOneColorKings,

    #[error("Invalid board: opponent is on check")]
    InvalidBoardOpponentIsOnCheck,

    #[error("Invalid board: en passant square does not have a pawn on it")]
    InvalidBoardInconsistentEnPassant,

    #[error("Invalid board: inconsistent castling rights")]
    InvalidBoardInconsistentCastlingRights,

    #[error("Illegal move detected")]
    IllegalMoveDetected,

    #[error("Chess move was not associated with the board")]
    NotAssociatedBoardMove,

    // Game Process Errors
    #[error("Illegal action detected")]
    IllegalActionDetected,

    #[error("Need to answer the draw offer")]
    DrawOfferNeedsAnswer,

    #[error("No draw offer detected")]
    DrawOfferNotDetected,

    #[error("Game is already finished")]
    GameIsAlreadyFinished,

    #[error("Wrong move number")]
    WrongMoveNumber,

    #[error("Invalid initialization PGN-string")]
    InvalidPGNString,
}
