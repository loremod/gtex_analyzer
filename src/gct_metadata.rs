use std::io;

#[derive(Debug)]
pub struct GCTMetadata {
    pub version: String,
    pub num_rows: usize,
    pub num_columns: usize,
    pub num_tissues: usize,
    pub column_names: Vec<String>,
}

impl GCTMetadata {
    pub fn new(
        version: String,
        num_rows: usize,
        num_columns: usize,
        num_tissues: usize,
        column_names: Vec<String>,
    ) -> Self {
        Self {
            version,
            num_rows,
            num_columns,
            num_tissues,
            column_names,
        }
    }

    pub fn get_tissue_names(&self) -> &[String] {
        &self.column_names[2..]
    }

    pub fn from_lines(mut lines: impl Iterator<Item = io::Result<String>>) -> io::Result<GCTMetadata>{
        // Read the first three lines
        let version = lines.next().unwrap_or(Ok(String::new()))?;
        let size_line = lines.next().unwrap_or(Ok(String::new()))?;
        let header_line = lines.next().unwrap_or(Ok(String::new()))?;
    
        let sizes: Vec<&str> = size_line.split_whitespace().collect();
        let num_rows = sizes.get(0).and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
        let num_tissues = sizes.get(1).and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
    
        let num_columns = num_tissues + 2;
        let column_names: Vec<String> = header_line.split_whitespace().map(|s| s.to_string()).collect();
        
        Ok(GCTMetadata::new(version, num_rows, num_columns, num_tissues, column_names))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gct_metadata_creation() {
        let metadata: GCTMetadata = GCTMetadata::new("v1.2".to_string(), 100, 50, 48, vec!["Sample1".to_string()]);
        assert_eq!(metadata.version, "v1.2");
        assert_eq!(metadata.num_rows, 100);
        assert_eq!(metadata.num_columns, 50);
        assert_eq!(metadata.num_tissues, 48);
        assert!(!metadata.column_names.is_empty());
    }

    #[test]
    fn test_gct_metadata_from_lines() {
        let input = vec![
            Ok("v1.2".to_string()),
            Ok("100 48".to_string()),
            Ok("ID SYMBOL Sample1 Sample2".to_string()),
        ];
        let metadata: GCTMetadata = GCTMetadata::from_lines(input.into_iter()).unwrap();
        assert_eq!(metadata.version, "v1.2");
        assert_eq!(metadata.num_rows, 100);
        assert_eq!(metadata.num_tissues, 48);
        assert_eq!(metadata.num_columns, 50);
    }
}
