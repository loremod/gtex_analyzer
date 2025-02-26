use gtex_analyzer::read_file;
use gtex_analyzer::{GtexSummary, GCTMetadata, GCTResults};
use std::fs::File;
use std::path::Path;
use flate2::read::GzDecoder;
use std::io::{self, BufRead, BufReader, Read};

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


#[test]
fn test_load() -> io::Result<()> {
    let file_path: &str = "data/GTEx_RNASeq_gene_median_tpm_HEAD.gct"; 

    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let summary: GtexSummary = read_file(reader, None)?;

    assert!(summary.metadata.is_some(), "Metadata should be present");
    assert!(!summary.results.get_results().is_empty(), "Results should not be empty");

    Ok(())
}


#[test]
fn test_on_sample_dataset() -> io::Result<()>{
    let file_path: &str = "../../../data/GTEx_RNASeq_gene_median_tpm_HEAD.gct"; // bulk Tissue Expression

    // let file_path: &str  = "../../../data/GTEx_Analysis_v10_RNASeQCv2.4.2_gene_median_tpm.gct.gz";
    // 1. Decode gz file
    let decoder = decode_file(file_path)?;
    // 2. Return an iterator of the file lines
    let reader = read_gct_file(decoder)?;

    let summary_wrap = read_file(reader, None);

    assert!(!summary_wrap.is_err(), "Expected an Ok(GtexSummary), not an Err");

    let summary = summary_wrap?;
    assert!(!summary.metadata.is_none(), "Expected GtexSummary to contain GCTMetadata, not None");
    assert!(!summary.results.get_results().is_empty(), "Expected GtexSummary to contain GCTResults with a populated HashMap, not empty");
    println!("{:#?}", summary.results);
    Ok(())
}