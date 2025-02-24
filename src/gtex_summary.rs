use crate::{GCTMetadata,GCTResults};
use std::io::{self, BufRead};

#[derive(Debug)]
pub struct GtexSummary {
    pub metadata: Option<GCTMetadata>,
    pub results: GCTResults,
    // results: HashMap<String, DGEResult>,
}

impl GtexSummary {
    pub fn new() -> Self{
        Self {
            metadata:None,
            results: GCTResults::new(),
            // results: HashMap::new(),
        }
    }

    pub fn from_reader<R: BufRead>(reader: R, n_max: Option<usize>) -> io::Result<Self> {
        let mut lines_iter = reader.lines(); // Iterator over lines
        let metadata = GCTMetadata::from_lines(&mut lines_iter)?;
        let results = GCTResults::from_rows(&mut lines_iter, &metadata, n_max);
        Ok(Self {
            metadata: Some(metadata),
            results,
        })
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_gtex_summary_from_reader() {
        let input_data = "v1.2\n100 48\nID SYMBOL Sample1 Sample2\nGene1 Symbol1 1.2 3.4\nGene2 Symbol2 1.2 3.4";
        let reader = Cursor::new(input_data);
        let summary = GtexSummary::from_reader(reader, None).unwrap();

        assert!(summary.metadata.is_some());
        let metadata = summary.metadata.as_ref().unwrap();
        assert_eq!(metadata.version, "v1.2");
        assert_eq!(metadata.num_tissues, 48);
        assert_eq!(metadata.num_columns, 48+2);
        assert_eq!(summary.results.get_results().len(), 2);
        assert!(summary.results.get_results().contains_key("Gene1"));
    }
}
