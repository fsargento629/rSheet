use std::error::Error;
use std::fs::File;
use std::path::Path;

/// Pure Domain representation of spreadsheet data.
/// Has zero dependency on Ratatui or terminal UI libraries.
pub struct Spreadsheet {
    pub data: Vec<Vec<String>>,
    pub max_rows: usize,
    pub max_cols: usize,
}

impl Spreadsheet {
    /// Create a spreadsheet with initialized empty strings.
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            data: vec![vec![String::new(); cols]; rows],
            max_rows: rows,
            max_cols: cols,
        }
    }

    /// Load a CSV file and pad/trim to fixed bounds.
    pub fn load_from_csv<P: AsRef<Path>>(
        path: P,
        max_rows: usize,
        max_cols: usize,
    ) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(file);

        let mut data = Vec::new();

        for record_result in rdr.records().take(max_rows) {
            let record = record_result?;
            let mut row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
            row.resize(max_cols, String::new());
            data.push(row);
        }

        while data.len() < max_rows {
            data.push(vec![String::new(); max_cols]);
        }

        Ok(Self {
            data,
            max_rows,
            max_cols,
        })
    }

    pub fn get_cell(&self, row: usize, col: usize) -> Option<&str> {
        self.data
            .get(row)
            .and_then(|r| r.get(col))
            .map(|s| s.as_str())
    }

    /// Convert column index to Excel-style letters (0 -> "A", 25 -> "Z", 26 -> "AA")
    pub fn col_to_letter(mut col: usize) -> String {
        let mut result = String::new();
        loop {
            let rem = (col % 26) as u8;
            result.insert(0, (b'A' + rem) as char);
            if col < 26 {
                break;
            }
            col = (col / 26) - 1;
        }
        result
    }
}
