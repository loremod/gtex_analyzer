use flate2::read::GzDecoder;
use gtex_analyzer::expression_analysis::GtexSummaryLoader;
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

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

fn compute_statistics(values: &[usize]) -> (f64, f64, usize, usize, f64) {
    if values.is_empty() {
        return (0.0, 0.0, 0, 0, 0.0);
    }

    let sum: usize = values.iter().sum();
    let count = values.len();
    let mean = sum as f64 / count as f64;

    let min = *values.iter().min().unwrap();
    let max = *values.iter().max().unwrap();

    let variance: f64 = values
        .iter()
        .map(|&x| {
            let diff = x as f64 - mean;
            diff * diff
        })
        .sum::<f64>()
        / count as f64;

    let std_dev = variance.sqrt();

    (mean, std_dev, min, max, variance)
}

fn main() -> io::Result<()> {
    // Parse command-line argument for output filename
    let filename = env::args()
        .nth(1)
        .expect("Usage: tissue_analysis <output_filename>");
    
    // Create results directory if it doesn't exist
    fs::create_dir_all("results")?;
    
    // Construct output file path
    let output_path = format!("results/{}.txt", filename);
    let mut output_file = File::create(&output_path)?;
    
    let file_path = "data/GTEx_Analysis_v10_RNASeQCv2.4.2_gene_median_tpm.gct.gz";

    writeln!(output_file, "Loading GTEx data from: {}", file_path)?;
    let reader = decode_file(file_path)?;

    writeln!(output_file, "Creating GtexSummaryLoader...")?;
    let summary_loader = GtexSummaryLoader::new(None, None);

    writeln!(output_file, "Loading summary (this may take a while for large files)...")?;
    let summary = summary_loader.load_summary(reader)?;

    writeln!(output_file, "\nAnalyzing tissue-level gene regulation...")?;

    // Get all tissue names from metadata
    let tissue_names = summary.metadata.get_tissue_names();
    let num_tissues = tissue_names.len();

    // Initialize counters for each tissue
    let mut up_regulated_counts: HashMap<String, usize> = HashMap::new();
    let mut down_regulated_counts: HashMap<String, usize> = HashMap::new();

    // Initialize all tissues with 0 counts
    for tissue_name in tissue_names.iter() {
        up_regulated_counts.insert(tissue_name.clone(), 0);
        down_regulated_counts.insert(tissue_name.clone(), 0);
    }

    // Count up-regulated and down-regulated genes for each tissue
    let results = summary.get_results();
    for dge_result in results.values() {
        // Count up-regulated tissues for this gene
        for tissue_analysis in &dge_result.up_regulated {
            *up_regulated_counts
                .get_mut(&tissue_analysis.tissue_name)
                .unwrap() += 1;
        }

        // Count down-regulated tissues for this gene
        for tissue_analysis in &dge_result.down_regulated {
            *down_regulated_counts
                .get_mut(&tissue_analysis.tissue_name)
                .unwrap() += 1;
        }
    }

    // Collect counts into vectors for statistics computation
    let up_counts: Vec<usize> = up_regulated_counts.values().copied().collect();
    let down_counts: Vec<usize> = down_regulated_counts.values().copied().collect();

    // Compute statistics
    let (up_mean, up_std, up_min, up_max, _) = compute_statistics(&up_counts);
    let (down_mean, down_std, down_min, down_max, _) = compute_statistics(&down_counts);

    // Print results
    writeln!(output_file, "\n=== Tissue-Level Gene Regulation Analysis ===")?;
    writeln!(output_file, "\nTotal number of genes analyzed: {}", results.len())?;
    writeln!(output_file, "Total number of tissues: {}", num_tissues)?;

    writeln!(output_file, "\n--- Up-Regulated Genes Statistics ---")?;
    writeln!(output_file, "  Total count across all tissues: {}", up_counts.iter().sum::<usize>())?;
    writeln!(output_file, "  Average per tissue: {:.2}", up_mean)?;
    writeln!(output_file, "  Standard deviation: {:.2}", up_std)?;
    writeln!(output_file, "  Minimum: {}", up_min)?;
    writeln!(output_file, "  Maximum: {}", up_max)?;

    writeln!(output_file, "\n--- Down-Regulated Genes Statistics ---")?;
    writeln!(output_file, "  Total count across all tissues: {}", down_counts.iter().sum::<usize>())?;
    writeln!(output_file, "  Average per tissue: {:.2}", down_mean)?;
    writeln!(output_file, "  Standard deviation: {:.2}", down_std)?;
    writeln!(output_file, "  Minimum: {}", down_min)?;
    writeln!(output_file, "  Maximum: {}", down_max)?;

    // Optional: Print per-tissue breakdown (commented out to avoid too much output)
    // Uncomment if you want to see individual tissue counts
    /*
    println!("\n--- Per-Tissue Breakdown ---");
    for tissue_name in tissue_names.iter() {
        let up_count = up_regulated_counts.get(tissue_name).unwrap();
        let down_count = down_regulated_counts.get(tissue_name).unwrap();
        println!(
            "  {}: Up-regulated: {}, Down-regulated: {}",
            tissue_name, up_count, down_count
        );
    }
    */

    Ok(())
}

