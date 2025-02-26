// use crate::models::{Metadata, Results};
use crate::{DGEResult, GCTMetadata, GCTResults};
use crate::TPMValue;
use std::collections::HashMap;
use std::io::{self, BufRead, ErrorKind};

#[derive(Debug)]
pub struct GtexSummary{
    pub metadata: GCTMetadata,
    results: GCTResults,
    // results: HashMap<String, DGEResult>,
}

impl GtexSummary {
    pub fn new(metadata: GCTMetadata, results: GCTResults) -> Self {
        Self {metadata, results}
    }

    pub fn from_reader<B: BufRead>(reader: B, n_max: Option<usize>) -> io::Result<Self> {
        let mut lines_iter = reader.lines(); // Iterator over lines
        let metadata = GCTMetadata::from_lines(&mut lines_iter)?;

        let data: Vec<_> = lines_iter
            .map(|x| x.expect("There should be no I/O-related issues during reading the file"))
            .collect();

        let results = GCTResults::from_rows(&mut data.iter().map(|x| x.as_str()), &metadata, n_max)?;
        Ok(Self {
            metadata: metadata,
            results,
        })
    }

    pub fn get_results(&self) -> &GCTResults {
        &self.results
    }
}

pub struct GtexSummaryLoader {
    n_max: Option<usize>,
    dge_threshold: Option<usize>
}

impl GtexSummaryLoader {
    pub fn new(n_max: Option<usize>, dge_threshold: Option<usize>) -> Self{
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

            let dge = parser.parse_row(&line?)?;

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

        let gctresults = GCTResults{results};

        Ok(GtexSummary::new(metadata, gctresults))
    }
}

pub struct RowParser<'a> {
    metadata: &'a GCTMetadata
}

impl<'a> RowParser<'a> {

    pub fn parse_row(&self, line: &str) -> io::Result<DGEResult> {
        // anyhow::bail!("I cannot proceed: {reason:?}")
        let (id, symbol, tpms) = Self::separate_id_symbol_tpm(line)?;
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
        let summary_wrap =
            GtexSummary::from_reader(reader, None);

        assert!(!summary_wrap.is_err());
        let summary = summary_wrap.unwrap();
        
        let metadata = summary.metadata;
        assert_eq!(metadata.version, "v1.2");
        assert_eq!(metadata.num_tissues, 2);
        assert_eq!(metadata.num_columns, 2 + 2);
        assert_eq!(metadata.column_names.len(), metadata.num_tissues);
        assert_eq!(summary.results.get_results().len(), 2);
        assert!(summary.results.get_results().contains_key("Gene1"));
    }
}
