
use std::io;


pub type ZScoreValue = f32;
pub type TPMValue = f32;

pub trait Metadata: Sized {
    fn from_lines(lines: &mut impl Iterator<Item = io::Result<String>>) -> io::Result<Self>;
}

pub trait Results<M: Metadata>: Sized {
    fn from_rows(
        rows: &mut impl Iterator<Item = std::io::Result<String>>,
        metadata: &M,
        n_max: Option<usize>
    ) -> std::io::Result<Self>;

    fn new() -> Self;
}
