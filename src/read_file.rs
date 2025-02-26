// use std::io::{self, BufRead};
// // use crate::{Metadata, Results};
// use crate::GtexSummary;


// // pub fn read_file<B: BufRead>(
// //     mut input: B,
// //     n_max: Option<usize>
// // ) -> io::Result<GtexSummary> {
    
// //     if input.fill_buf()?.is_empty() {
// //         return Err(io::Error::new(io::ErrorKind::InvalidInput, "The file is empty"));
// //     }

// //     GtexSummary::from_reader(input, n_max)
// // }


// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::io::Cursor;
//     use crate::{GCTMetadata,GCTResults};
    
//     #[test]
//     fn test_empty_file_returns_error() {
//         let empty_input = Cursor::new(Vec::new()); // Simulates an empty file

//         let result: Result<GtexSummary, std::io::Error> = read_file(empty_input, None);

//         assert!(result.is_err(), "Expected an error for an empty file.");
//         if let Err(e) = result {
//             assert!(e.to_string().contains("The file is empty"));
//         }
//     }
// }
