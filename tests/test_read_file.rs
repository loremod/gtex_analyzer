use gtex_analyzer::{read_file, GtexSummaryLoader};
use gtex_analyzer::{GtexSummary, GCTMetadata};
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

    let summary_loader = GtexSummaryLoader::new(Some(10),  None);
    let summary = summary_loader.load_summary(reader)?;

    // println!("{:#?}", summary.get_results());

    // assert!(summary.metadata.is_some(), "Metadata should be present");
    assert!(!summary.get_results().is_empty(), "Results should not be empty");

    Ok(())
}


#[test]
fn test_on_sample_dataset() -> io::Result<()>{
    let file_path: &str = "data/GTEx_RNASeq_gene_median_tpm_HEAD.gct"; // bulk Tissue Expression

    // let file_path: &str  = "../../../data/GTEx_Analysis_v10_RNASeQCv2.4.2_gene_median_tpm.gct.gz";
    // 1. Decode gz file
    let decoder = decode_file(file_path)?;
    // 2. Return an iterator of the file lines
    let reader = read_gct_file(decoder)?;

    let summary_loader = GtexSummaryLoader::new(Some(10),  None);
    let summary = summary_loader.load_summary(reader)?;



    // assert!(!summary.metadata.is_none(), "Expected GtexSummary to contain GCTMetadata, not None");
    assert!(summary.metadata.num_tissues > 0);
    assert_eq!(summary.metadata.num_columns, summary.metadata.num_tissues + 2);
    assert_eq!(summary.metadata.column_names.len(), summary.metadata.num_columns);
    assert!(!summary.get_results().is_empty(), "Expected GtexSummary to contain GCTResults with a populated HashMap, not empty");

    //Add more specifict assertions
    Ok(())
}