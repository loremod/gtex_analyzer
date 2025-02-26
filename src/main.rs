use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;
use flate2::read::GzDecoder; //  decompression of gz
use gtex_analyzer::{read_file, GtexSummary};
use gtex_analyzer::{GCTMetadata, GCTResults};
use gtex_analyzer::ZScoreValue;
use gtex_analyzer::TPMValue;
use gtex_analyzer::GtexSummaryLoader;


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
fn study_dataset(file_path: &str, n_max: Option<usize>) -> io::Result<()> {

    let decoder = decode_file(file_path)?;

    // 2. Return an iterator of the file lines
    let reader  = read_gct_file(decoder)?;
    let mut lines_iter  = reader.lines();
    let metadata: GCTMetadata = GCTMetadata::from_lines(&mut lines_iter)?;

    let mut results: HashMap<String, (String, f32, f32, f32, f32)> = HashMap::new();

    for (index, line) in lines_iter.enumerate() {
        if let Some(limit) = n_max {
            if index >= limit {
                break;
            }
        }

        if let Ok(content) = line {
            let (id, symbol, tpms) = GCTResults::separate_id_symbol_tpm(&content)?;

            let mean: TPMValue = tpms.iter().copied().sum::<TPMValue>() / tpms.len() as TPMValue;
            let variance: TPMValue = tpms.iter()
                                    .map(|x| (x-mean).powi(2))
                                    .sum::<TPMValue>() / tpms.len() as TPMValue;
    
            let sd: TPMValue = variance.sqrt();
            let min = tpms.iter().cloned().fold(f32::INFINITY, f32::min);
            let max = tpms.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

            results.insert(id.to_string(), (symbol.to_string(), mean, sd, min, max));
        }
    }

    for (id, (symbol, mean, std, min, max)) in &results {
        println!("Gene: {}, Symbol: {}, Mean: {:.3}, Std: {:.3}, Min: {:.3}, Max: {:.3}", id, symbol, mean, std, min, max);
    }

    Ok(())
}



fn main()  -> io::Result<()>{
    let file_path: &str = "../../../data/GTEx_RNASeq_gene_median_tpm_HEAD.gct"; // bulk Tissue Expression
    
    
    // study_dataset(file_path, None)?;

    
    
    
    // let file_path: &str  = "../../../data/GTEx_Analysis_v10_RNASeQCv2.4.2_gene_median_tpm.gct.gz";
    // 1. Decode gz file
    let decoder = decode_file(file_path)?;
    // 2. Return an iterator of the file lines
    let reader = read_gct_file(decoder)?;

    let summary_loader = GtexSummaryLoader::new(Some(10),  None);
    let summary = summary_loader.load_summary(reader)?;

    println!("{:#?}", summary.get_results());
    


    // println!("{:#?}", summary);
    // println!("{}",summary.metadata.as_ref().unwrap().num_columns);
    // println!(summary.metadata.);
    Ok(())
}

