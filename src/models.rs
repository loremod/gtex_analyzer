

// fn read_gct_gz_file<R: Read>(decoder: R) -> io::Result<impl Iterator<Item = io::Result<String>>>{
//     let reader = io::BufReader::new(decoder);
//     Ok(reader.lines())
// }

//-----EDIT-----

pub type ZScoreValue = f32;
pub type TPMValue = f32;
