use std::io::{self, BufRead};

use crate::GtexSummary;


pub fn read_file<R: BufRead>(mut input: R) -> io::Result<GtexSummary> {
    
    if input.fill_buf()?.is_empty() {
        eprintln!("Warning: The file is empty.");
        return Ok(GtexSummary::new());
    }

    GtexSummary::from_reader(input, None)
}

// CLAP