use crate::ZScoreValue;
use crate::TPMValue;
use crate::GCTMetadata;

#[derive(Debug)]
pub struct DGEResult{
    pub id: String, // referred to as Name
    pub symbol: String, // referred to as Description
    pub up_regulated: Vec<TissueAnalysis>, // pair<TissueName, ZScoreValue>
    pub down_regulated: Vec<TissueAnalysis>,// pair<TissueName, ZScoreValue>
}

#[derive(Debug)]
pub struct TissueAnalysis {
    pub tissue_name: String,
    pub z_score: ZScoreValue, // maybe I'm exaggerating with the type dimension, to do: ask the Professor
}

impl DGEResult{
    pub fn new(id: String, symbol: String) -> Self{
        Self {  id: id,
                symbol: symbol,
                up_regulated: Vec::new(),
                down_regulated: Vec::new(),
            }
    }

    pub fn add_up_regulated(&mut self, tissue_name: String, z_score:ZScoreValue){
        self.up_regulated.push(TissueAnalysis{tissue_name: tissue_name, z_score: z_score});
    }

    pub fn add_down_regulated(&mut self, tissue_name: String, z_score: ZScoreValue) {
        self.down_regulated.push(TissueAnalysis {tissue_name: tissue_name, z_score: z_score });
    }

    pub fn perform_analysis(&mut self, tpms: &[TPMValue], metadata: &GCTMetadata) {
        let tissue_names: &[String] = metadata.get_tissue_names();
        
        let mean: TPMValue = tpms.iter().copied().sum::<TPMValue>() / tpms.len() as TPMValue;
        let variance: TPMValue = tpms.iter()
                                .map(|x| (x-mean).powi(2))
                                .sum::<TPMValue>() / tpms.len() as TPMValue;

        let sd: TPMValue = variance.sqrt();

        for (tissue, &tpm_value) in tissue_names.iter().zip(tpms.iter()) {
            let zscore = (tpm_value - mean) / sd;
            if zscore >= 2.0 {
                self.add_up_regulated(tissue.clone(), zscore); 
            } else if zscore <= -2.0 {
                self.add_down_regulated(tissue.clone(), zscore); 
            }
        }
    }
}