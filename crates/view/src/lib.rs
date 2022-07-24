#![feature(once_cell)]

mod field;
mod segment;
mod view_path;

pub use field::{Field, FieldBuf};
pub use segment::{Segment, SegmentBuf};
pub use view_path::{ViewPath, ViewPathBuf};

pub mod parser;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid path {0}")]
    InvalidPath(String),
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
