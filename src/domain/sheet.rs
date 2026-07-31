use std::error::Error;
use std::fs::File;
use std::path::Path;

/// Pure Domain representation of spreadsheet data.
pub struct Spreadsheet {
    pub data: Vec<Vec<String>>,
    pub max_rows: usize,
    pub max_cols: usize,
    pub loaded_path: Option<String>,
}

impl Spreadsheet {
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            data: vec![vec![String::new(); cols]; rows],
            max_rows: rows,
            max_cols: cols,
            loaded_path: None,
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(
        path: P,
        max_rows: usize,
        max_cols: usize,
    ) -> Result<Self, Box<dyn Error>> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        let file = File::open(&path)?;
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
            loaded_path: Some(path_str),
        })
    }

    pub fn get_cell(&self, row: usize, col: usize) -> Option<&str> {
        self.data
            .get(row)
            .and_then(|r| r.get(col))
            .map(|s| s.as_str())
    }

    pub fn set_cell(&mut self, row: usize, col: usize, value: String) {
        if row < self.max_rows && col < self.max_cols {
            self.data[row][col] = value;
        }
    }

    /// Save current spreadsheet grid back to CSV file (trims trailing empty rows/cols).
    pub fn save_to_csv<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::create(path)?;
        let mut wtr = csv::WriterBuilder::new()
            .has_headers(false)
            .from_writer(file);

        // Find actual non-empty row bound
        let mut last_row = 0;
        for r in (0..self.max_rows).rev() {
            if self.data[r].iter().any(|c| !c.is_empty()) {
                last_row = r;
                break;
            }
        }

        // Find actual non-empty col bound
        let mut last_col = 0;
        for row in &self.data {
            for c in (0..self.max_cols).rev() {
                if !row[c].is_empty() {
                    if c > last_col {
                        last_col = c;
                    }
                    break;
                }
            }
        }

        for r in 0..=last_row {
            let slice = &self.data[r][0..=last_col];
            wtr.write_record(slice)?;
        }

        wtr.flush()?;
        Ok(())
    }

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
