use gtex_analyzer::read_file;
use gtex_analyzer::{GtexSummary, GCTMetadata, GCTResults};
use std::fs::File;
use std::io::{self, BufReader};

#[test]
fn test_load() -> io::Result<()> {
    let file_path: &str = "data/GTEx_RNASeq_gene_median_tpm_HEAD.gct"; 

    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let summary: GtexSummary<GCTMetadata, GCTResults> = read_file(reader)?;

    assert!(summary.metadata.is_some(), "Metadata should be present");
    assert!(!summary.results.get_results().is_empty(), "Results should not be empty");

    Ok(())
}