// use crate::models::{Metadata, Results};
use crate::{DGEResult, GCTMetadata, ZScoreValue};
use crate::TPMValue;
use std::collections::HashMap;
use std::io::{self, BufRead, Error, ErrorKind};

#[derive(Debug)]
pub struct GtexSummary{
    pub metadata: GCTMetadata,
    results: HashMap<String, DGEResult>,
    // results: HashMap<String, DGEResult>,
}

impl GtexSummary {
    pub fn new(metadata: GCTMetadata, results: HashMap<String, DGEResult>) -> Self {
        Self {metadata, results}
    }

    // pub fn from_reader<B: BufRead>(reader: B, n_max: Option<usize>) -> io::Result<Self> {
    //     let mut lines_iter = reader.lines(); // Iterator over lines
    //     let metadata = GCTMetadata::from_lines(&mut lines_iter)?;

    //     let data: Vec<_> = lines_iter
    //         .map(|x| x.expect("There should be no I/O-related issues during reading the file"))
    //         .collect();

    //     let results = GCTResults::from_rows(&mut data.iter().map(|x| x.as_str()), &metadata, n_max)?;
    //     Ok(Self {
    //         metadata: metadata,
    //         results,
    //     })
    // }

    pub fn get_results(&self) -> &HashMap<String, DGEResult> {
        &self.results
    }
}

pub struct GtexSummaryLoader {
    n_max: Option<usize>,
    dge_threshold: Option<ZScoreValue>
}

impl GtexSummaryLoader {
    pub fn new(n_max: Option<usize>, dge_threshold: Option<ZScoreValue>) -> Self{
        Self{n_max, dge_threshold}
    }

    pub fn load_summary<B>(&self, data: B) -> io::Result<GtexSummary>
    where
        B: BufRead,
    {
        let mut lines = data.lines();
        // (1) parse the metadata to get the number of columns
        //   create the metadata
        let metadata = GCTMetadata::from_lines(&mut lines)?;

        // (2) parse the records
        let parser = RowParser {metadata: &metadata};

        let mut results = HashMap::new();
        
        for (index, line) in lines.enumerate() {
            // TODO: possibly use `n_max` here to break out
            if let Some(max_index) = self.n_max{
                if index == max_index{
                    break;
                }
            }

            let dge = parser.parse_row(&line?, index)?;

            // Check if the ID is already present
            match results.entry(dge.id.to_string()) {
                std::collections::hash_map::Entry::Occupied(_) => {
                    return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, 
                        format!("Row with ID (Name) '{}' already exists", dge.id)));
                }
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert(dge);
                }
            }
        }


        Ok(GtexSummary::new(metadata, results))
    }
}

pub struct RowParser<'a> {
    metadata: &'a GCTMetadata
}

impl<'a> RowParser<'a> {

    pub fn parse_row(&self, line: &str, index: usize) -> io::Result<DGEResult> {
        // anyhow::bail!("I cannot proceed: {reason:?}")
        let (id, symbol, tpms) = Self::separate_id_symbol_tpm(line)?;

        if tpms.len() != self.metadata.num_tissues {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "Invalid number of tpm values with respect to the header, row number {}.\nExpected values: {}, found: {}.\nThis row will be skipped.",
                    index + 4, self.metadata.num_tissues, tpms.len()
                ),
            ));
        }

        //create DGEResult
        let dge_result = DGEResult::from_analysis(id.to_string(), symbol.to_string(), &tpms, self.metadata);
        Ok(dge_result)
    }

    // Splits a line into ID, Symbol, and TPM values
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
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_gtex_summary_from_reader() {
        let input_data =
            "v1.2\n100 2\nID SYMBOL Sample1 Sample2\nGene1 Symbol1 1.2 3.4\nGene2 Symbol2 1.2 3.4";
        let reader = Cursor::new(input_data);

        let summary_loader = GtexSummaryLoader::new(Some(10),  None);
        let summary_wrap = summary_loader.load_summary(reader);
        assert!(!summary_wrap.is_err());


        let summary = summary_wrap.unwrap();
        let metadata = &summary.metadata;
        assert_eq!(metadata.version, "v1.2");
        assert_eq!(metadata.num_tissues, 2);
        assert_eq!(metadata.num_columns, 2 + 2);
        assert_eq!(metadata.column_names.len(), metadata.num_columns);
        assert_eq!(summary.get_results().len(), 2);
        assert!(summary.get_results().contains_key("Gene1"));
    }

    #[test]
    fn test_separate_id_symbol_tpm() {
        let content = "Gene1 Symbol1 1.2 3.4 5.6";
        let output = RowParser::separate_id_symbol_tpm(content);
        
        assert!(!output.is_err(), "It should not return an Err");

        let (id, symbol, tpms) = output.expect("It should separate correctly");

        assert_eq!(id, "Gene1");
        assert_eq!(symbol, "Symbol1");
        assert_eq!(tpms.len(), 3);
    }

    #[test]
    fn test_from_rows() -> Result<(), Box<dyn std::error::Error>> {
        let input = vec![
            "v1.0\n3 3\n ID SYMBOL T1 T2 T3".to_string(),
            "Gene1 Symbol1 1.2 3.4 5.6".to_string(),
            "Gene2 Symbol2 2.2 4.4 6.6".to_string(),
            "Gene3 Symbol2 2.2 4.4 6.6".to_string(),
        ];

        let summary_loader = GtexSummaryLoader::new(None, Some(1.2));
        let input_data = input.join("\n");
        let cursor = Cursor::new(input_data.into_bytes());
        let risultati = summary_loader.load_summary(cursor);
        assert!(!risultati.is_err(), "It should not return an Err");
        assert_eq!(risultati?.get_results().len(), 3);
        Ok(())
    }

    #[test]
    fn test_from_rows_with_n_max() -> Result<(), Box<dyn std::error::Error>>{
        let input = vec![
            "v1.0\n3 3\n ID SYMBOL T1 T2 T3".to_string(),
            "Gene1 Symbol1 1.2 3.4 5.6".to_string(),
            "Gene2 Symbol2 2.2 4.4 6.6".to_string(),
            "Gene3 Symbol2 2.2 4.4 6.6".to_string(),
        ];
        let summary_loader = GtexSummaryLoader::new(Some(1), Some(1.2));
        let input_data = input.join("\n");
        let cursor = Cursor::new(input_data.into_bytes());
        let partial_results = summary_loader.load_summary(cursor);
        assert!(!partial_results.is_err(), "It should not return an Err");
        assert_eq!(partial_results?.get_results().len(), 1);
        Ok(())
    }

    #[test]
    fn test_correct_tpm_list_length() -> Result<(), Box<dyn std::error::Error>>{
        let input = vec![
            "Gene1 Symbol1 1.2 3.4 5.6".to_string(),
            "Gene2 Symbol2 2.2 4.4 ".to_string(),
            "Gene3 Symbol2 2.2 4.4 6.6".to_string(),
        ];
        let summary_loader = GtexSummaryLoader::new(None, Some(1.2));
        let input_data = input.join("\n");
        let cursor = Cursor::new(input_data.into_bytes());
        let result = summary_loader.load_summary(cursor);
        assert!(result.is_err());
        let unwrapped_result = result.unwrap_err();
        println!("{}",unwrapped_result.to_string());
        // assert!(unwrapped_result.to_string().contains("Invalid number of tpm values"));
        Ok(())
    }

    // #[test]
    // fn test_() -> Result<(), Box<dyn std::error::Error>>{
    //     let input = vec![
    //         "Gene1 Symbol1 1.2 3.4 5.6".to_string(),
    //         "Gene1 Symbol1 2.2 4.4 6.6".to_string(),
    //         "Gene3 Symbol2 22.2 14.4 16.6".to_string(),
    //     ];
    //     let metadata = GCTMetadata::new("v1.0".to_string(), 2, 5, 3, vec!["ID".to_string(), "SYMBOL".to_string()]);
    //     let result = GCTResults::from_rows(input.iter().map(|s| s.as_str()), &metadata, Some(3));
    //     assert!(result.is_err());
    //     assert!(result.unwrap_err().to_string().contains("already exists"));
    //     Ok(())
    // }
}
