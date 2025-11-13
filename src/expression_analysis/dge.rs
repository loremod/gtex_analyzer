use super::GCTMetadata;
use super::TPMValue;
use super::ZScoreValue;
use serde::{Serialize, Deserialize};

/// Stores statistical information about the gene's differential expression across tissues.
///
/// It stores the Gene ID, the Gene symbol and a Vector of up_regulated and down_regulated tissues
#[derive(Debug, Serialize, Deserialize)]
pub struct DGEResult {
    pub id: String,                          // referred to as Name
    pub symbol: String,                      // referred to as Description
    pub up_regulated: Vec<TissueAnalysis>,   // pair<TissueName, ZScoreValue>
    pub down_regulated: Vec<TissueAnalysis>, // pair<TissueName, ZScoreValue>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TissueAnalysis {
    pub tissue_name: String,
    /// Z-scores for expression levels in the specific tissues with respect to all tissues.
    /// Computed on log2-transformed TPM values.
    pub z_score: ZScoreValue,
    /// Fold change relative to median expression across all tissues.
    /// Computed as: tpm_value / median_tpm
    pub fold_change: TPMValue,
}

impl DGEResult {
    pub fn new(id: String, symbol: String) -> Self {
        Self {
            id,
            symbol,
            up_regulated: Vec::new(),
            down_regulated: Vec::new(),
        }
    }
    pub fn add_up_regulated(&mut self, tissue_name: String, z_score: ZScoreValue, fold_change: TPMValue) {
        self.up_regulated.push(TissueAnalysis {
            tissue_name,
            z_score,
            fold_change,
        });
    }

    pub fn add_down_regulated(&mut self, tissue_name: String, z_score: ZScoreValue, fold_change: TPMValue) {
        self.down_regulated.push(TissueAnalysis {
            tissue_name,
            z_score,
            fold_change,
        });
    }

    /// Computes differentially expressed genes based on log2-transformed Z-scores and fold change.
    /// 
    /// This method:
    /// 1. Filters out genes with maximum TPM < 1.0 (minimum expression threshold)
    /// 2. Applies log2 transformation with pseudocount (TPM + 0.1) for robust statistics
    /// 3. Computes z-scores on log2-transformed values
    /// 4. Computes fold change relative to median expression
    /// 
    /// # Arguments
    /// * `tpms` - Raw TPM values across all tissues
    /// * `metadata` - GCT metadata containing tissue names
    /// * `dge_threshold` - Z-score threshold for differential expression (typically 2.0)
    /// 
    /// # Returns
    /// Returns `true` if the gene was analyzed (passed minimum expression filter), `false` otherwise.
    pub fn perform_analysis(
        &mut self,
        tpms: &[TPMValue],
        metadata: &GCTMetadata,
        dge_threshold: ZScoreValue,
    ) -> bool {
        let tissue_names: &[String] = metadata.get_tissue_names();

        // Minimum expression filter: skip genes with max TPM < 1.0
        let max_tpm = tpms.iter().copied().fold(0.0, TPMValue::max);
        if max_tpm < 1.0 {
            return false; // Gene filtered out due to low expression
        }

        // Log2 transformation with pseudocount to avoid log(0)
        // Using pseudocount of 0.1 (common practice: 0.1-1.0 TPM)
        const PSEUDOCOUNT: TPMValue = 0.1;
        let log_tpms: Vec<TPMValue> = tpms
            .iter()
            .map(|&tpm| (tpm + PSEUDOCOUNT).log2())
            .collect();

        // Compute statistics on log2-transformed values
        let mean: TPMValue = log_tpms.iter().copied().sum::<TPMValue>() / log_tpms.len() as TPMValue;
        let variance: TPMValue = log_tpms
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<TPMValue>()
            / log_tpms.len() as TPMValue;

        let sd: TPMValue = variance.sqrt();

        // Compute median for fold change calculation (on original TPM values)
        let mut sorted_tpms: Vec<TPMValue> = tpms.to_vec();
        sorted_tpms.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median_tpm = if sorted_tpms.len() % 2 == 0 {
            let mid = sorted_tpms.len() / 2;
            (sorted_tpms[mid - 1] + sorted_tpms[mid]) / 2.0
        } else {
            sorted_tpms[sorted_tpms.len() / 2]
        };

        // Avoid division by zero for fold change
        let median_tpm = median_tpm.max(PSEUDOCOUNT);

        for (tissue, (&tpm_value, &log_tpm)) in tissue_names.iter().zip(tpms.iter().zip(log_tpms.iter())) {
            let zscore = if sd > 0.0 {
                (log_tpm - mean) / sd
            } else {
                0.0 // If all values are the same, z-score is 0
            };

            let fold_change = tpm_value / median_tpm;

            if zscore >= dge_threshold {
                self.add_up_regulated(tissue.clone(), zscore, fold_change);
            } else if zscore <= -dge_threshold {
                self.add_down_regulated(tissue.clone(), zscore, fold_change);
            }
        }

        true // Gene was analyzed
    }

    pub fn from_analysis(
        id: String,
        symbol: String,
        tpms: &[TPMValue],
        metadata: &GCTMetadata,
        dge_threshold: ZScoreValue,
    ) -> Self {
        let mut dgeresult = Self::new(id.to_string(), symbol.to_string());
        dgeresult.perform_analysis(tpms, metadata, dge_threshold);
        dgeresult
    }
}
