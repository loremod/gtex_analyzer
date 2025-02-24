use std::io::{self, BufRead};

use crate::GtexSummary;


pub fn read_file<R: BufRead>(input: R) -> io::Result<GtexSummary> {
    GtexSummary::from_reader(input, None)
}

// CLAP