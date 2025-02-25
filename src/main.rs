use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;
use flate2::read::GzDecoder; //  decompression of gz
use gtex_analyzer::{read_file, GtexSummary};
use gtex_analyzer::{GCTMetadata, GCTResults};

// fn read_gct_gz_file<R: Read>(decoder: R) -> io::Result<impl Iterator<Item = io::Result<String>>>{
//     let reader = io::BufReader::new(decoder);
//     Ok(reader.lines())
// }







//------ FOR THE MAIN -------
// fn decode(file_path: &str) -> io::Result<impl Read> {
//     let path = Path::new(file_path);
//     let file = File::open(path)?; // Open the .gz file
//     let decoder: GzDecoder<File> = GzDecoder::new(file); // Decompress it
//     Ok(decoder)
// }

fn decode_file(file_path: &str) -> io::Result<Box<dyn BufRead>> {
    let path = Path::new(file_path);
    let file = File::open(path)?;

    // Check if the file is a `.gz` and decode it if necessary
    if file_path.ends_with(".gz") {
        let decoder = GzDecoder::new(file);
        Ok(Box::new(BufReader::new(decoder)))
    } else {
        Ok(Box::new(BufReader::new(file)))
    }
}

fn read_gct_file<R: Read>(decoder: R) -> io::Result<BufReader<R>>{
    let reader = io::BufReader::new(decoder);
    Ok(reader)
}

fn main()  -> io::Result<()>{
    let file_path: &str = "../../../data/GTEx_RNASeq_gene_median_tpm_HEAD.gct"; // bulk Tissue Expression
    // let file_path: &str  = "../../../data/GTEx_Analysis_v10_RNASeQCv2.4.2_gene_median_tpm.gct.gz";
    // 1. Decode gz file
    let decoder = decode_file(file_path)?;
    // 2. Return an iterator of the file lines
    let reader = read_gct_file(decoder)?;

    let summary: GtexSummary<GCTMetadata, GCTResults> = read_file(reader)?;

    // println!("{:#?}", summary);
    // println!("{}",summary.metadata.as_ref().unwrap().num_columns);
    // println!(summary.metadata.);
    println!("{:#?}", summary.results);

    Ok(())
}

