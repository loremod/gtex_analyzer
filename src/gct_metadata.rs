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
        let version = lines.next().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Not enough metadata lines."))??;
        let size_line = lines.next().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Not enough metadata lines."))??;
        let header_line = lines.next().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Not enough metadata lines."))??;
    
        let sizes: Vec<&str> = size_line.split_whitespace().collect();
        if sizes.len() < 2 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid size line format. Expected at least two values."));
        }
        let num_rows = sizes[0].parse::<usize>().map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid row count format"))?;
        let num_tissues = sizes[1].parse::<usize>().map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid tissue count format"))?;
    
        let num_columns = num_tissues + 2;
        let column_names: Vec<String> = header_line.split_whitespace().map(|s| s.to_string()).collect();
        
        if column_names.len() != num_columns {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!(
                "Invalid header length. Expected {} columns, but found {}.",
                num_columns, column_names.len()
            )));
        }

        Ok(Self{version, num_rows, num_columns, num_tissues, column_names})
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
            Ok("100 2".to_string()),
            Ok("ID SYMBOL Sample1 Sample2".to_string()),
        ];
        let metadata: GCTMetadata = GCTMetadata::from_lines(input.into_iter()).unwrap();
        assert_eq!(metadata.version, "v1.2");
        assert_eq!(metadata.num_rows, 100);
        assert_eq!(metadata.num_tissues, 2);
        assert_eq!(metadata.num_columns, 4);
    }

    #[test]
    fn test_missing_metadata_lines() {
        let input = vec![
            // Ok("v1.2".to_string()),
            Ok("100 2".to_string()),
            Ok("ID SYMBOL Sample1 Sample2".to_string()),
        ];
        let result = GCTMetadata::from_lines(input.into_iter());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Not enough metadata lines.");
    }

    #[test]
    fn test_invalid_size_format() {
        let input = vec![
            Ok("v1.2".to_string()),
            Ok("100".to_string()),
            Ok("ID SYMBOL Sample1 Sample2".to_string()),
        ];
        let result = GCTMetadata::from_lines(input.into_iter());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Invalid size line format. Expected at least two values.");
    }

    #[test]
    fn test_invalid_header_column_length() {
        let input = vec![
            Ok("v1.2".to_string()),
            Ok("100 2".to_string()),
            Ok("ID SYMBOL Sample1 ".to_string()),
        ];
        let result = GCTMetadata::from_lines(input.into_iter());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid header length"));
    }
}
