use super::GCTMetadata;
use super::TPMValue;
use super::ZScoreValue;

/// Stores statistical information about the gene's differential expression across tissues.
///
/// It stores the Gene ID, the Gene symbol and a Vector of up_regulated and down_regulated tissues
#[derive(Debug)]
pub struct DGEResult {
    pub id: String,                          // referred to as Name
    pub symbol: String,                      // referred to as Description
    pub up_regulated: Vec<TissueAnalysis>,   // pair<TissueName, ZScoreValue>
    pub down_regulated: Vec<TissueAnalysis>, // pair<TissueName, ZScoreValue>
}

#[derive(Debug)]
pub struct TissueAnalysis {
    pub tissue_name: String,
    /// Z-scores for expression levels in the specific tissues with respect to all tissues.
    pub z_score: ZScoreValue,
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
    pub fn add_up_regulated(&mut self, tissue_name: String, z_score: ZScoreValue) {
        self.up_regulated.push(TissueAnalysis {
            tissue_name,
            z_score,
        });
    }

    pub fn add_down_regulated(&mut self, tissue_name: String, z_score: ZScoreValue) {
        self.down_regulated.push(TissueAnalysis {
            tissue_name,
            z_score,
        });
    }

    /// It compute differentially expressed genes based on Z-scores.
    pub fn perform_analysis(
        &mut self,
        tpms: &[TPMValue],
        metadata: &GCTMetadata,
        dge_threshold: ZScoreValue,
    ) {
        let tissue_names: &[String] = metadata.get_tissue_names();

        let mean: TPMValue = tpms.iter().copied().sum::<TPMValue>() / tpms.len() as TPMValue;
        let variance: TPMValue =
            tpms.iter().map(|x| (x - mean).powi(2)).sum::<TPMValue>() / tpms.len() as TPMValue;

        let sd: TPMValue = variance.sqrt();

        for (tissue, &tpm_value) in tissue_names.iter().zip(tpms.iter()) {
            let zscore = (tpm_value - mean) / sd;
            if zscore >= dge_threshold {
                self.add_up_regulated(tissue.clone(), zscore);
            } else if zscore <= -dge_threshold {
                self.add_down_regulated(tissue.clone(), zscore);
            }
        }
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
