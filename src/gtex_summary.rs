use crate::models::{Metadata, Results};
use crate::{DGEResult, GCTMetadata, GCTResults};
use std::io::{self, BufRead};

#[derive(Debug)]
pub struct GtexSummary<M: Metadata, R: Results<M>> {
    pub metadata: Option<M>,
    pub results: R,
    // results: HashMap<String, DGEResult>,
}

impl<M: Metadata, R: Results<M>> GtexSummary<M, R> {
    pub fn new() -> Self {
        Self {
            metadata: None,
            results: R::new(),
            // results: HashMap::new(),
        }
    }

    pub fn from_reader<B: BufRead>(reader: B, n_max: Option<usize>) -> io::Result<Self> {
        let mut lines_iter = reader.lines(); // Iterator over lines
        let metadata = M::from_lines(&mut lines_iter)?;

        let data: Vec<_> = lines_iter
            .map(|x| x.expect("There should be no I/O-related issues during reading the file"))
            .collect();

        let results = R::from_rows(&mut data.iter().map(|x| x.as_str()), &metadata, n_max)?;
        Ok(Self {
            metadata: Some(metadata),
            results,
        })
    }
}

pub struct GtexSummaryLoader {
    n_max: Option<usize>,
}

impl GtexSummaryLoader {
    pub fn load_summary<M, R, B>(&self, data: B) -> anyhow::Result<GtexSummary<M, R>>
    where
        M: Metadata,
        R: Results<M>,
        B: BufRead,
    {
        // (1) parse the metadata to get the number of columns
        //   create the metadata
        let metadata = todo!();
        // (2) parse the records
        let parser = RowParser {metadata: &metadata};


        for line in data.lines() {
            // TODO: possibly use `n_max` here to break out
            let dge = parser.parse_row(&line?)?;
        }

        
        todo!()
    }
}

pub struct RowParser<'a> {
    metadata: &'a GCTMetadata
}

impl<'a> RowParser<'a> {

    pub fn parse_row(&self, line: &str) -> anyhow::Result<DGEResult> {
        if false {
            let reason = "bla";
            anyhow::bail!("I cannot proceed: {reason:?}")
        }
        todo!()
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
        let summary: GtexSummary<GCTMetadata, GCTResults> =
            GtexSummary::from_reader(reader, None).unwrap();

        assert!(summary.metadata.is_some());
        let metadata = summary.metadata.as_ref().unwrap();
        assert_eq!(metadata.version, "v1.2");
        assert_eq!(metadata.num_tissues, 2);
        assert_eq!(metadata.num_columns, 2 + 2);
        assert_eq!(summary.results.get_results().len(), 2);
        assert!(summary.results.get_results().contains_key("Gene1"));
    }
}
