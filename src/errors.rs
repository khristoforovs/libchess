use failure::Fail;

#[derive(Debug, Clone, Fail)]
pub enum ChessBoardCoordinatesError {
    #[fail(display = "Invalid index for board's file: {}", n)]
    InvalidBoardFileIndex { n: usize },

    #[fail(display = "Negative file index found")]
    NegativeBoardFileIndex,

    #[fail(display = "Invalid board's file string representation")]
    InvalidBoardFileName,

    #[fail(display = "Invalid index for board's rank: {}", n)]
    InvalidBoardRankIndex { n: usize },

    #[fail(display = "Negative rank index found")]
    NegativeBoardRankIndex,

    #[fail(display = "Invalid board's rank string representation")]
    InvalidBoardRankName,

    #[fail(display = "Invalid square representation string")]
    InvalidSquareRepresentation,
}

#[derive(Debug, Clone, Fail)]
pub enum PieceRepresentationError {
    #[fail(display = "Invalid peace representation string")]
    InvalidPeaceRepresentation,

    #[fail(display = "Invalid peace index : {}", n)]
    InvalidPeaceIndex { n: usize },

    #[fail(display = "Invalid color index : {}", n)]
    InvalidColorIndex { n: usize },
}

#[derive(Debug, Clone, Fail)]
pub enum ChessBoardError {
    #[fail(display = "Invalid FEN string: {}", s)]
    InvalidFENString { s: String },

    #[fail(display = "Invalid position: colors overlapping detected")]
    InvalidPositionColorsOverlap,

    #[fail(display = "Invalid position: 2 or more piece type overlap detected")]
    InvalidPositionPieceTypeOverlap,

    #[fail(display = "Invalid board: combined mask is not self-consistent")]
    InvalidBoardSelfNonConsistency,

    #[fail(display = "Invalid board: more than 1 king of the same color")]
    InvalidBoardMultipleOneColorKings,

    #[fail(display = "Invalid board: opponent is on check")]
    InvalidBoardOpponentIsOnCheck,

    #[fail(display = "Invalid board: en passant square does not have a pawn on it")]
    InvalidBoardInconsistentEnPassant,

    #[fail(display = "Invalid board: inconsistent castling rights")]
    InvalidBoardInconsistentCastlingRights,

    #[fail(display = "Illegal move detected")]
    IllegalMoveDetected,
}

#[derive(Debug, Clone, Fail)]
pub enum GameError {
    #[fail(display = "Illegal action detected")]
    IllegalActionDetected,

    #[fail(display = "Need to answer the draw offer")]
    DrawOfferNeedsAnswer,

    #[fail(display = "No draw offer detected")]
    DrawOfferNotDetected,

    #[fail(display = "Game is already finished")]
    GameIsAlreadyFinished,
}
