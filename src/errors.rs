use failure::Fail;

#[derive(Debug, Clone, Fail)]
pub enum Error {
    #[fail(display="Invalid index for board's file: {}", n)]
    InvalidBoardFileIndex {n: i64},

    #[fail(display="Invalid board's file string representation")]
    InvalidBoardFileName,

    #[fail(display="Invalid index for board's rank: {}", n)]
    InvalidBoardRankIndex {n: i64},

    #[fail(display="Invalid board's rank string representation")]
    InvalidBoardRankName,
}