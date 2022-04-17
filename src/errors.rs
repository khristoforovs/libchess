use failure::Fail;

#[derive(Debug, Clone, Fail)]
pub enum Error {
    // Board coordinates errors
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

    // Piece representation errors
    #[fail(display = "Invalid peace representation string")]
    InvalidPeaceRepresentation,

    #[fail(display = "Invalid peace index : {}", n)]
    InvalidPeaceIndex { n: usize },

    #[fail(display = "Invalid color index : {}", n)]
    InvalidColorIndex { n: usize },

    // Board Builder errors
    #[fail(display = "Invalid FEN string: {}", s)]
    InvalidFENString { s: String },
}
