use failure::Fail;

#[derive(Debug, Clone, Fail)]
pub enum Error {
    #[fail(display="Invalid index for board's file: {}", n)]
    InvalidBoardFileIndex {n: i64},
}