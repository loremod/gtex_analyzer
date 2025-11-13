use flate2::read::GzDecoder;
use gtex_analyzer::expression_analysis::GtexSummaryLoader;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

fn decode_file(file_path: &str) -> io::Result<Box<dyn BufRead>> {
    let path = Path::new(file_path);
    let file = File::open(path)?;

    if file_path.ends_with(".gz") {
        let decoder = GzDecoder::new(file);
        Ok(Box::new(BufReader::new(decoder)))
    } else {
        Ok(Box::new(BufReader::new(file)))
    }
}

fn analyze_threshold(
    file_path: &str,
    threshold: f32,
    sample_size: Option<usize>,
) -> io::Result<()> {
    println!("\n=== Analyzing with z-score threshold: {:.1} ===", threshold);
    
    let reader = decode_file(file_path)?;
    let summary_loader = GtexSummaryLoader::new(sample_size, Some(threshold));
    let summary = summary_loader.load_summary(reader)?;

    let results = summary.get_results();
    
    // Collect fold changes for up and down regulated genes
    let mut up_fold_changes: Vec<f32> = Vec::new();
    let mut down_fold_changes: Vec<f32> = Vec::new();
    let mut up_z_scores: Vec<f32> = Vec::new();
    let mut down_z_scores: Vec<f32> = Vec::new();
    
    let mut up_count = 0;
    let mut down_count = 0;

    for dge_result in results.values() {
        for tissue in &dge_result.up_regulated {
            up_fold_changes.push(tissue.fold_change);
            up_z_scores.push(tissue.z_score);
            up_count += 1;
        }
        for tissue in &dge_result.down_regulated {
            down_fold_changes.push(tissue.fold_change);
            down_z_scores.push(tissue.z_score);
            down_count += 1;
        }
    }

    // Compute statistics
    if !up_fold_changes.is_empty() {
        up_fold_changes.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let up_median_fc = up_fold_changes[up_fold_changes.len() / 2];
        let up_p25_fc = up_fold_changes[up_fold_changes.len() / 4];
        let up_p75_fc = up_fold_changes[3 * up_fold_changes.len() / 4];
        let up_min_fc = up_fold_changes[0];
        let up_max_fc = up_fold_changes[up_fold_changes.len() - 1];
        
        up_z_scores.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let up_median_z = up_z_scores[up_z_scores.len() / 2];
        let up_min_z = up_z_scores[0];
        let up_max_z = up_z_scores[up_z_scores.len() - 1];

        println!("\nUp-regulated genes: {}", up_count);
        println!("  Fold change - Median: {:.2}x, Q1: {:.2}x, Q3: {:.2}x, Min: {:.2}x, Max: {:.2}x", 
                 up_median_fc, up_p25_fc, up_p75_fc, up_min_fc, up_max_fc);
        println!("  Z-score - Median: {:.2}, Min: {:.2}, Max: {:.2}", 
                 up_median_z, up_min_z, up_max_z);
    }

    if !down_fold_changes.is_empty() {
        down_fold_changes.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let down_median_fc = down_fold_changes[down_fold_changes.len() / 2];
        let down_p25_fc = down_fold_changes[down_fold_changes.len() / 4];
        let down_p75_fc = down_fold_changes[3 * down_fold_changes.len() / 4];
        let down_min_fc = down_fold_changes[0];
        let down_max_fc = down_fold_changes[down_fold_changes.len() - 1];
        
        down_z_scores.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let down_median_z = down_z_scores[down_z_scores.len() / 2];
        let down_min_z = down_z_scores[0];
        let down_max_z = down_z_scores[down_z_scores.len() - 1];

        println!("\nDown-regulated genes: {}", down_count);
        println!("  Fold change - Median: {:.2}x, Q1: {:.2}x, Q3: {:.2}x, Min: {:.2}x, Max: {:.2}x", 
                 down_median_fc, down_p25_fc, down_p75_fc, down_min_fc, down_max_fc);
        println!("  Z-score - Median: {:.2}, Min: {:.2}, Max: {:.2}", 
                 down_median_z, down_min_z, down_max_z);
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let file_path = "data/GTEx_Analysis_v10_RNASeQCv2.4.2_gene_median_tpm.gct.gz";
    
    println!("Analyzing different z-score thresholds");
    println!("Using a sample of 5000 genes for faster analysis");
    println!("================================================");

    // Test different thresholds
    let thresholds = vec![1.0, 1.5, 1.96, 2.0, 2.5, 3.0];
    
    for threshold in thresholds {
        analyze_threshold(file_path, threshold, Some(5000))?;
    }

    println!("\n\n=== Summary ===");
    println!("Z-score thresholds tested: 1.0, 1.5, 1.96, 2.0, 2.5, 3.0");
    println!("\nInterpretation:");
    println!("- Z-score 1.0: ~84th percentile (moderate)");
    println!("- Z-score 1.5: ~93rd percentile (moderate-high)");
    println!("- Z-score 1.96: ~95th percentile (standard confidence interval)");
    println!("- Z-score 2.0: ~97.7th percentile (high, current default)");
    println!("- Z-score 2.5: ~99.4th percentile (very high)");
    println!("- Z-score 3.0: ~99.9th percentile (extremely high)");
    println!("\nNote: With 68 tissues, a z-score of 2.0 means the expression");
    println!("is in the top ~2.3% for that gene across all tissues.");

    Ok(())
}

