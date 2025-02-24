use crate::TPMValue;
use crate::{DGEResult, GCTMetadata};
use std::collections::HashMap;
use std::io;

#[derive(Debug)]
pub struct GCTResults {
    results: HashMap<String, DGEResult>,
}

impl GCTResults {
    /// Creates a new empty `GCTResults`
    pub fn new() -> Self {
        Self {
            results: HashMap::new(),
        }
    }

    /// Creates a `GCTResults` from an iterator over rows
    pub fn from_rows(rows: impl Iterator<Item = io::Result<String>>, metadata: &GCTMetadata, n_max: Option<usize>) -> Self {
        let mut results = HashMap::new(); // HashMap to store analyzed gene products
        for (index, line) in rows.enumerate() {
            if let Some(limit) = n_max{
                if index >= limit {
                    break;
                }
            }
            if let Ok(content) = line {
                let (id, symbol, tpms) = Self::separate_id_symbol_tpm(&content);

                // Check if the ID is already present
                match results.entry(id.to_string()) {
                    std::collections::hash_map::Entry::Occupied(_) => {
                        println!("Row with ID (Name) '{}' already exists!", id);
                    }
                    std::collections::hash_map::Entry::Vacant(entry) => {
                        let dge_result = entry.insert(DGEResult::new(id.to_string(), symbol.to_string()));
                        dge_result.perform_analysis(&tpms, metadata);
                    }
                }
            }
        }

        Self { results }
    }

    /// Splits a line into ID, Symbol, and TPM values
    pub fn separate_id_symbol_tpm(content: &str) -> (&str, &str, Box<[TPMValue]>) {
        let elems: Vec<&str> = content.split_whitespace().collect();
        let id: &str = elems[0];
        let symbol: &str = elems[1];
        let tpms: Box<[TPMValue]> = elems[2..]
            .iter()
            .map(|elem| elem.parse::<TPMValue>().expect("Failed to parse"))
            .collect::<Vec<TPMValue>>()
            .into_boxed_slice();
        (id, symbol, tpms)
    }

    /// Returns a reference to results
    pub fn get_results(&self) -> &HashMap<String, DGEResult> {
        &self.results
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_separate_id_symbol_tpm() {
        let content = "Gene1 Symbol1 1.2 3.4 5.6";
        let (id, symbol, tpms) = GCTResults::separate_id_symbol_tpm(content);

        assert_eq!(id, "Gene1");
        assert_eq!(symbol, "Symbol1");
        assert_eq!(tpms.len(), 3);
    }

    #[test]
    fn test_from_rows() {
        let input = vec![
            Ok("Gene1 Symbol1 1.2 3.4 5.6".to_string()),
            Ok("Gene2 Symbol2 2.2 4.4 6.6".to_string()),
            Ok("Gene3 Symbol2 2.2 4.4 6.6".to_string()),
        ];
        let metadata = GCTMetadata::new("v1.0".to_string(), 2, 5, 3, vec!["ID".to_string(), "SYMBOL".to_string()]);
        let risultati = GCTResults::from_rows(input.into_iter(), &metadata, None);
        assert_eq!(risultati.get_results().len(), 3);
    }

    #[test]
    fn test_from_rows_with_n_max(){
        let input = vec![
            Ok("Gene1 Symbol1 1.2 3.4 5.6".to_string()),
            Ok("Gene2 Symbol2 2.2 4.4 6.6".to_string()),
            Ok("Gene3 Symbol2 2.2 4.4 6.6".to_string()),
        ];
        let metadata = GCTMetadata::new("v1.0".to_string(), 2, 5, 3, vec!["ID".to_string(), "SYMBOL".to_string()]);
        let partial_results = GCTResults::from_rows(input.into_iter(), &metadata, Some(1));
        assert_eq!(partial_results.get_results().len(), 1);
    }
}
