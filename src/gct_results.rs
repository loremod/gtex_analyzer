use crate::TPMValue;
use crate::{DGEResult, GCTMetadata};
use std::collections::HashMap;
use std::io::{self, Error, ErrorKind};
use crate::Results;

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
    pub fn from_rows<'a>(rows: impl Iterator<Item = &'a str>, metadata: &GCTMetadata, n_max: Option<usize>) -> io::Result<Self> {
        let mut results = HashMap::new(); // HashMap to store analyzed gene products
        for (index, line) in rows.enumerate() {
            if let Some(limit) = n_max{
                if index >= limit {
                    break;
                }
            }
            
            let (id, symbol, tpms) = Self::separate_id_symbol_tpm(line)?;

            // Check if the number of TPM values is equal to the number of columns of the tissues
            if tpms.len() != metadata.num_tissues {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!(
                        "Invalid number of tpm values with respect to the header, row number {}.\nExpected values: {}, found: {}.\nThis row will be skipped.",
                        index + 4, metadata.num_tissues, tpms.len()
                    ),
                ));
            }

            // Check if the ID is already present
            match results.entry(id.to_string()) {
                std::collections::hash_map::Entry::Occupied(_) => {
                    return Err(Error::new(ErrorKind::InvalidInput,
                            format!("Row with ID (Name) '{}' already exists!", id)));
                }
                std::collections::hash_map::Entry::Vacant(entry) => {
                    let dge_result = entry.insert(DGEResult::new(id.to_string(), symbol.to_string()));
                    dge_result.perform_analysis(&tpms, metadata);
                }
            }
        }

        Ok(Self { results })
    }

    /// Splits a line into ID, Symbol, and TPM values
    pub fn separate_id_symbol_tpm(content: &str) -> io::Result<(&str, &str, Box<[TPMValue]>)> {
        let elems: Vec<&str> = content.split_whitespace().collect();
        let id: &str = elems[0];
        let symbol: &str = elems[1];
        let tpms: Box<[TPMValue]> = elems[2..]
        .iter()
        .map(|elem| elem.parse::<TPMValue>().map_err(|_| io::Error::new(
            io::ErrorKind::InvalidData, 
            format!("Invalid TPM value for gene ID {}: '{}'", id , elem)
        )))
        .collect::<Result<Vec<TPMValue>, io::Error>>()?
        .into_boxed_slice();
        Ok((id, symbol, tpms))
    }

    /// Returns a reference to results
    pub fn get_results(&self) -> &HashMap<String, DGEResult> {
        &self.results
    }
}


impl Results<GCTMetadata> for GCTResults {
    fn from_rows<'a>(
        rows: &mut impl Iterator<Item = &'a str>,
        metadata: &GCTMetadata,
        n_max: Option<usize>
    ) -> io::Result<Self> {
        GCTResults::from_rows(rows, metadata, n_max)
    }

    fn new() -> Self {
        GCTResults { results: std::collections::HashMap::new() }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_separate_id_symbol_tpm() {
        let content = "Gene1 Symbol1 1.2 3.4 5.6";
        let (id, symbol, tpms) = GCTResults::separate_id_symbol_tpm(content).expect("It should separate correctly");

        assert_eq!(id, "Gene1");
        assert_eq!(symbol, "Symbol1");
        assert_eq!(tpms.len(), 3);
    }

    #[test]
    fn test_from_rows() -> Result<(), Box<dyn std::error::Error>> {
        let input = vec![
            "Gene1 Symbol1 1.2 3.4 5.6".to_string(),
            "Gene2 Symbol2 2.2 4.4 6.6".to_string(),
            "Gene3 Symbol2 2.2 4.4 6.6".to_string(),
        ];

        let metadata = GCTMetadata::new("v1.0".to_string(), 2, 5, 3, vec!["ID".to_string(), "SYMBOL".to_string()]);
        let risultati = GCTResults::from_rows(input.iter().map(|s| s.as_str()), &metadata, None)?;
        assert_eq!(risultati.get_results().len(), 3);
        Ok(())
    }

    #[test]
    fn test_from_rows_with_n_max() -> Result<(), Box<dyn std::error::Error>>{
        let input = vec![
            "Gene1 Symbol1 1.2 3.4 5.6".to_string(),
            "Gene2 Symbol2 2.2 4.4 6.6".to_string(),
            "Gene3 Symbol2 2.2 4.4 6.6".to_string(),
        ];
        let metadata = GCTMetadata::new("v1.0".to_string(), 2, 5, 3, vec!["ID".to_string(), "SYMBOL".to_string()]);
        let partial_results = GCTResults::from_rows(input.iter().map(|s| s.as_str()), &metadata, Some(1))?;
        assert_eq!(partial_results.get_results().len(), 1);
        Ok(())
    }

    #[test]
    fn test_correct_tpm_list_length() -> Result<(), Box<dyn std::error::Error>>{
        let input = vec![
            "Gene1 Symbol1 1.2 3.4 5.6".to_string(),
            "Gene2 Symbol2 2.2 4.4 ".to_string(),
            "Gene3 Symbol2 2.2 4.4 6.6".to_string(),
        ];
        let metadata = GCTMetadata::new("v1.0".to_string(), 2, 5, 3, vec!["ID".to_string(), "SYMBOL".to_string()]);
        let result = GCTResults::from_rows(input.iter().map(|s| s.as_str()), &metadata, Some(3));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid number of tpm values"));
        Ok(())
    }

    #[test]
    fn test_() -> Result<(), Box<dyn std::error::Error>>{
        let input = vec![
            "Gene1 Symbol1 1.2 3.4 5.6".to_string(),
            "Gene1 Symbol1 2.2 4.4 6.6".to_string(),
            "Gene3 Symbol2 22.2 14.4 16.6".to_string(),
        ];
        let metadata = GCTMetadata::new("v1.0".to_string(), 2, 5, 3, vec!["ID".to_string(), "SYMBOL".to_string()]);
        let result = GCTResults::from_rows(input.iter().map(|s| s.as_str()), &metadata, Some(3));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
        Ok(())
    }
}
