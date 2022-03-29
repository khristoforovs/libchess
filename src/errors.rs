use failure::Fail;

#[derive(Debug, Clone, Fail)]
pub enum Error {
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

    #[fail(display = "Invalid peace representation string")]
    InvalidPeaceRepresentation,
}
