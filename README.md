# GTEx Analyzer

A Rust library for analyzing GTEx (Genotype-Tissue Expression) gene expression data. This library provides tools to load, process, and analyze RNA-seq expression data from GTEx datasets, with a focus on identifying differentially expressed genes (DGE) across tissues.

## Features

- **GCT File Support**: Read and parse GTEx GCT (Gene Cluster Text) format files, including gzipped files
- **Differential Expression Analysis**: Identify up-regulated and down-regulated genes across tissues using robust statistical methods
- **Metadata Handling**: Extract and work with dataset metadata (version, tissue counts, column names)
- **Efficient Processing**: Handle large datasets with optional row limits for testing and development
- **Serialization Support**: Save and load analysis results in JSON or binary (bincode) formats
- **Log2 Transformation**: Uses log2-transformed expression values for more robust statistical analysis
- **Fold Change Calculation**: Computes fold change relative to median expression for biological interpretability

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
gtex_analyzer = { path = "." }
```

## Quick Start

### Loading and Analyzing GTEx Data

```rust
use flate2::read::GzDecoder;
use gtex_analyzer::expression_analysis::GtexSummaryLoader;
use std::fs::File;
use std::io::BufReader;

fn main() -> std::io::Result<()> {
    // Open and decode the gzipped GCT file
    let file = File::open("data/GTEx_Analysis_v10_RNASeQCv2.4.2_gene_median_tpm.gct.gz")?;
    let decoder = GzDecoder::new(file);
    let reader = BufReader::new(decoder);

    // Create a loader with default settings (no row limit, default threshold of 2.0)
    let summary_loader = GtexSummaryLoader::new(None, None);
    
    // Load and analyze the data
    let summary = summary_loader.load_summary(reader)?;

    // Access results
    let results = summary.get_results();
    println!("Analyzed {} genes", results.len());

    // Iterate through differentially expressed genes
    for (gene_id, dge_result) in results {
        println!("Gene: {} ({})", gene_id, dge_result.symbol);
        println!("  Up-regulated in {} tissues", dge_result.up_regulated.len());
        println!("  Down-regulated in {} tissues", dge_result.down_regulated.len());
    }

    Ok(())
}
```

### Using Custom Thresholds

```rust
// Limit to first 1000 genes and use z-score threshold of 1.5
let summary_loader = GtexSummaryLoader::new(Some(1000), Some(1.5));
```

### Saving and Loading Results

```rust
// Save to JSON
summary.save_json("results/analysis.json")?;

// Save to binary format (faster, smaller)
summary.save_bincode("results/analysis.bin")?;

// Load from binary
let loaded_summary = GtexSummary::load_bincode("results/analysis.bin")?;
```

## Command-Line Tools

### Tissue Analysis Script

Analyze tissue-level gene regulation statistics:

```bash
cargo run --bin tissue_analysis output_filename
```

This will:
- Load the full GTEx dataset
- Count up-regulated and down-regulated genes per tissue
- Compute statistics (mean, std dev, min, max) across all tissues
- Save results to `results/output_filename.txt`

Example output:
```
=== Tissue-Level Gene Regulation Analysis ===

Total number of genes analyzed: 59033
Total number of tissues: 68

--- Up-Regulated Genes Statistics ---
  Total count across all tissues: 63144
  Average per tissue: 928.59
  Standard deviation: 1393.39
  Minimum: 88
  Maximum: 10495

--- Down-Regulated Genes Statistics ---
  Total count across all tissues: 30435
  Average per tissue: 447.57
  Standard deviation: 1081.86
  Minimum: 0
  Maximum: 6981
```

## Differential Expression Analysis Methodology

This section explains how differentially expressed genes (DGE) are identified, including the evolution from the initial approach to the current robust implementation.

### Current Implementation (Recommended)

The current implementation uses best practices from RNA-seq analysis to provide more robust and biologically meaningful results.

#### Step-by-Step Process

1. **Minimum Expression Filter**
   - Genes with maximum TPM < 1.0 across all tissues are filtered out
   - This removes noise from very lowly expressed genes that may give spurious results
   - Only genes with meaningful expression are analyzed

2. **Log2 Transformation with Pseudocount**
   - Raw TPM values are transformed: `log2(TPM + 0.1)`
   - The pseudocount (0.1) prevents issues with zero values
   - Log transformation is essential because:
     - Expression changes are multiplicative, not additive
     - The distribution of expression values is highly skewed
     - Log transformation makes the data more normally distributed
     - Reduces the impact of extreme outliers

3. **Z-Score Calculation on Log-Transformed Data**
   - Mean and standard deviation are computed on log2-transformed values
   - Z-score: `z = (log2_tpm - mean_log2) / sd_log2`
   - This provides a standardized measure of how unusual a gene's expression is in a tissue

4. **Fold Change Calculation**
   - Fold change is computed on original (non-transformed) TPM values
   - Formula: `fold_change = tpm_value / median_tpm`
   - Median is used as it's more robust to outliers than mean
   - Provides a biologically interpretable measure (e.g., 2.0 = 2x expression)

5. **Differential Expression Classification**
   - **Up-regulated**: `z_score >= threshold` (default: 2.0)
   - **Down-regulated**: `z_score <= -threshold` (default: -2.0)
   - Both z-score and fold change are stored for each tissue

#### Code Example

```rust
// The analysis is performed automatically when loading data
let summary_loader = GtexSummaryLoader::new(None, Some(2.0)); // z-score threshold of 2.0
let summary = summary_loader.load_summary(reader)?;

// Access results with both z-scores and fold changes
for dge_result in summary.get_results().values() {
    for tissue in &dge_result.up_regulated {
        println!(
            "{}: z-score={:.2}, fold-change={:.2}x",
            tissue.tissue_name, tissue.z_score, tissue.fold_change
        );
    }
}
```

### Previous Implementation (Deprecated)

The original implementation computed z-scores directly on raw TPM values without transformation, which led to several issues:

#### Original Process

1. **No Expression Filtering**
   - All genes were analyzed regardless of expression level
   - Very lowly expressed genes could produce spurious results

2. **Raw TPM Z-Scores**
   - Z-scores computed directly on raw TPM values
   - Formula: `z = (tpm - mean_tpm) / sd_tpm`
   - No log transformation applied

3. **No Fold Change**
   - Only z-scores were computed
   - No fold change information available

#### Problems with Original Approach

1. **Extreme Asymmetry**
   - Results showed 121,251 up-regulated vs 985 down-regulated genes (123:1 ratio)
   - This extreme imbalance was largely an artifact of the statistical method, not biology

2. **Skewed Distribution**
   - TPM values are highly right-skewed (many low values, few very high values)
   - Computing statistics on raw values gives disproportionate weight to outliers
   - Mean and standard deviation are heavily influenced by a few very high values

3. **Poor Down-Regulation Detection**
   - With raw TPM values, it's very difficult for a gene to be "exceptionally low"
   - If most tissues have low expression, the mean is already low, leaving little room to be "very low"
   - This led to almost no down-regulated genes being detected

4. **Statistical Issues**
   - Z-scores on raw data don't follow normal distribution assumptions
   - Variance is dominated by high-expression outliers
   - Results are not comparable across genes with different expression levels

### Results Comparison

The improvements in methodology led to dramatically more balanced and biologically meaningful results:

| Metric | Original (Raw TPM) | Current (Log2-Transformed) | Improvement |
|--------|-------------------|-------------------------|-------------|
| **Up-regulated genes** | 121,251 total<br>(avg 1,783/tissue) | 63,144 total<br>(avg 929/tissue) | More conservative, realistic |
| **Down-regulated genes** | 985 total<br>(avg 14/tissue) | 30,435 total<br>(avg 448/tissue) | **30x more detected** |
| **Up/Down ratio** | 123:1 | 2:1 | Much more balanced |
| **Up-regulation std dev** | 2,477 | 1,393 | More consistent across tissues |
| **Down-regulation std dev** | 66 | 1,082 | Better tissue variation detection |

### Why Log Transformation Matters

1. **Biological Reality**: Gene expression changes are multiplicative. A 2-fold increase from 10 to 20 TPM is biologically similar to 2-fold from 100 to 200 TPM, but on raw scale they're very different.

2. **Statistical Robustness**: Log transformation makes the data more normally distributed, which is important for z-score calculations and makes results more comparable across genes.

3. **Outlier Handling**: Extreme expression values (e.g., 10,000 TPM) have less disproportionate influence after log transformation.

4. **Better Down-Regulation Detection**: On log scale, being "low" is more symmetric with being "high", allowing better detection of tissue-specific down-regulation.

### Best Practices Applied

The current implementation follows RNA-seq analysis best practices:

- ✅ Log transformation (log2) before statistical analysis
- ✅ Minimum expression threshold to filter noise
- ✅ Median-based fold change for robustness
- ✅ Pseudocount to handle zero values
- ✅ Both statistical (z-score) and biological (fold change) metrics

## Data Structures

### `GtexSummary`

Main container for analysis results:

```rust
pub struct GtexSummary {
    pub metadata: GCTMetadata,
    results: HashMap<String, DGEResult>,
}
```

### `DGEResult`

Stores differential expression information for a single gene:

```rust
pub struct DGEResult {
    pub id: String,
    pub symbol: String,
    pub up_regulated: Vec<TissueAnalysis>,
    pub down_regulated: Vec<TissueAnalysis>,
}
```

### `TissueAnalysis`

Information about expression in a specific tissue:

```rust
pub struct TissueAnalysis {
    pub tissue_name: String,
    pub z_score: ZScoreValue,      // Z-score on log2-transformed data
    pub fold_change: TPMValue,      // Fold change relative to median
}
```

## File Format

The library expects GTEx data in GCT (Gene Cluster Text) format:

```
#1.2
<num_genes> <num_tissues>
Name    Description    <tissue1>    <tissue2>    ...
<gene_id>    <symbol>    <tpm1>    <tpm2>    ...
```

Example:
```
#1.2
59033 68
Name    Description    Adipose_Subcutaneous    Adipose_Visceral    ...
ENSG00000223972    DDX11L1    0.0    0.0    ...
```

## Testing

Run the test suite:

```bash
cargo test
```

## License

[Add your license here]

## Contributing

[Add contribution guidelines here]

## References

- GTEx Portal: https://gtexportal.org/
- GCT File Format: https://software.broadinstitute.org/cancer/software/gsea/wiki/index.php/Data_formats#GCT:_Gene_Cluster_Text_file_format
- RNA-seq Best Practices: https://www.nature.com/articles/nrg2484

