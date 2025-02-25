use std::io::{self, BufRead};
use crate::{Metadata, Results};
use crate::GtexSummary;


pub fn read_file<B: BufRead, M: Metadata, R: Results<M>>(
    mut input: B,
    n_max: Option<usize>
) -> io::Result<GtexSummary<M, R>> {
    
    if input.fill_buf()?.is_empty() {
        eprintln!("Warning: The file is empty.");
        return Ok(GtexSummary::new());
    }

    GtexSummary::from_reader(input, n_max)
}
