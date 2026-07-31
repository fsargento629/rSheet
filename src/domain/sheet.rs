use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum CellValue {
    Text(String),
    Number(f64),
    Error(CellError),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CellError {
    Syntax,
    DivByZero,
    Circular,
    Ref,
    Value,
}

impl CellError {
    pub fn as_str(&self) -> &'static str {
        match self {
            CellError::Syntax => "#SYNTAX!",
            CellError::DivByZero => "#DIV/0!",
            CellError::Circular => "#CIRCULAR!",
            CellError::Ref => "#REF!",
            CellError::Value => "#VALUE!",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Cell {
    pub raw: String,
    pub computed: CellValue,
}

impl Cell {
    pub fn new(raw: String) -> Self {
        let mut cell = Self {
            raw,
            computed: CellValue::Text(String::new()),
        };
        cell.evaluate_static();
        cell
    }

    pub fn evaluate_static(&mut self) {
        let trimmed = self.raw.trim();
        if trimmed.starts_with('=') {
            return;
        }

        if trimmed.is_empty() {
            self.computed = CellValue::Text(String::new());
        } else if let Ok(num) = trimmed.parse::<f64>() {
            self.computed = CellValue::Number(num);
        } else {
            self.computed = CellValue::Text(trimmed.to_string());
        }
    }

    pub fn display_text(&self) -> String {
        match &self.computed {
            CellValue::Text(s) => s.clone(),
            CellValue::Number(n) => {
                if n.fract() == 0.0 {
                    format!("{:.0}", n)
                } else {
                    format!("{:.2}", n)
                }
            }
            CellValue::Error(err) => err.as_str().to_string(),
        }
    }
}

pub struct Spreadsheet {
    pub data: Vec<Vec<Cell>>,
    pub max_rows: usize,
    pub max_cols: usize,
    pub loaded_path: Option<String>,
    /// Maps a cell to the set of cells it DEPENDS ON: cell -> { dependencies }
    pub dependencies: HashMap<(usize, usize), HashSet<(usize, usize)>>,
    /// Maps a cell to the set of cells that DEPEND ON IT: cell -> { dependents }
    pub dependents: HashMap<(usize, usize), HashSet<(usize, usize)>>,
}

impl Spreadsheet {
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            data: vec![vec![Cell::new(String::new()); cols]; rows],
            max_rows: rows,
            max_cols: cols,
            loaded_path: None,
            dependencies: HashMap::new(),
            dependents: HashMap::new(),
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
            let mut row: Vec<Cell> = record.iter().map(|s| Cell::new(s.to_string())).collect();
            while row.len() < max_cols {
                row.push(Cell::new(String::new()));
            }
            data.push(row);
        }

        while data.len() < max_rows {
            data.push(vec![Cell::new(String::new()); max_cols]);
        }

        let mut sheet = Self {
            data,
            max_rows,
            max_cols,
            loaded_path: Some(path_str),
            dependencies: HashMap::new(),
            dependents: HashMap::new(),
        };

        sheet.rebuild_graph_and_evaluate_all();
        Ok(sheet)
    }

    pub fn set_cell(&mut self, row: usize, col: usize, raw: String) {
        if row >= self.max_rows || col >= self.max_cols {
            return;
        }

        let trimmed = raw.trim();
        let is_formula = trimmed.starts_with('=');
        let new_refs = if is_formula {
            self.extract_references(&trimmed[1..])
        } else {
            HashSet::new()
        };

        // 1. Check if adding these edges creates a circular reference
        if is_formula && self.creates_cycle((row, col), &new_refs) {
            self.clear_dependencies((row, col));
            self.data[row][col].raw = raw;
            self.data[row][col].computed = CellValue::Error(CellError::Circular);
            // Propagate error downstream
            self.propagate_changes((row, col));
            return;
        }

        // 2. Update DAG edges
        self.clear_dependencies((row, col));
        self.data[row][col].raw = raw;

        if !new_refs.is_empty() {
            self.dependencies.insert((row, col), new_refs.clone());
            for dep in new_refs {
                self.dependents.entry(dep).or_default().insert((row, col));
            }
        }

        // 3. Re-evaluate this cell and all downstream dependents in topological order
        self.propagate_changes((row, col));
    }

    pub fn get_cell(&self, row: usize, col: usize) -> Option<&Cell> {
        self.data.get(row).and_then(|r| r.get(col))
    }

    /// Checks if making `target` depend on `proposed_deps` creates a cycle in the DAG using DFS
    fn creates_cycle(
        &self,
        target: (usize, usize),
        proposed_deps: &HashSet<(usize, usize)>,
    ) -> bool {
        if proposed_deps.contains(&target) {
            return true;
        }

        let mut stack: Vec<(usize, usize)> = proposed_deps.iter().cloned().collect();
        let mut visited = HashSet::new();

        while let Some(current) = stack.pop() {
            if current == target {
                return true;
            }
            if visited.insert(current) {
                if let Some(downstream) = self.dependencies.get(&current) {
                    for &next_dep in downstream {
                        stack.push(next_dep);
                    }
                }
            }
        }

        false
    }

    fn clear_dependencies(&mut self, cell: (usize, usize)) {
        if let Some(old_deps) = self.dependencies.remove(&cell) {
            for old_dep in old_deps {
                if let Some(dep_set) = self.dependents.get_mut(&old_dep) {
                    dep_set.remove(&cell);
                }
            }
        }
    }

    /// Recomputes `start` and all cells that depend on it in Topological Sort Order (Kahn's Algorithm)
    fn propagate_changes(&mut self, start: (usize, usize)) {
        let order = self.get_topological_dependents(start);
        for (r, c) in order {
            self.recompute_cell(r, c);
        }
    }

    /// Returns all cells reachable from `start` in topological order (dependencies evaluated first)
    fn get_topological_dependents(&self, start: (usize, usize)) -> Vec<(usize, usize)> {
        let mut reachable = HashSet::new();
        let mut queue = vec![start];

        while let Some(curr) = queue.pop() {
            if reachable.insert(curr) {
                if let Some(deps) = self.dependents.get(&curr) {
                    for &next_cell in deps {
                        queue.push(next_cell);
                    }
                }
            }
        }

        // Compute in-degrees on the reachable subgraph
        let mut in_degree: HashMap<(usize, usize), usize> = HashMap::new();
        for &cell in &reachable {
            in_degree.entry(cell).or_insert(0);
            if let Some(deps) = self.dependents.get(&cell) {
                for &next_cell in deps {
                    if reachable.contains(&next_cell) {
                        *in_degree.entry(next_cell).or_insert(0) += 1;
                    }
                }
            }
        }

        let mut ready: Vec<(usize, usize)> = reachable
            .iter()
            .cloned()
            .filter(|&c| in_degree.get(&c) == Some(&0))
            .collect();

        let mut sorted = Vec::new();
        while let Some(cell) = ready.pop() {
            sorted.push(cell);
            if let Some(deps) = self.dependents.get(&cell) {
                for &next_cell in deps {
                    if reachable.contains(&next_cell) {
                        if let Some(deg) = in_degree.get_mut(&next_cell) {
                            *deg -= 1;
                            if *deg == 0 {
                                ready.push(next_cell);
                            }
                        }
                    }
                }
            }
        }

        if !sorted.contains(&start) {
            sorted.insert(0, start);
        }

        sorted
    }

    fn recompute_cell(&mut self, row: usize, col: usize) {
        let raw = self.data[row][col].raw.clone();
        let trimmed = raw.trim();
        if trimmed.starts_with('=') {
            let res = self.eval_expression(&trimmed[1..]);
            self.data[row][col].computed = res;
        } else {
            self.data[row][col].evaluate_static();
        }
    }

    pub fn rebuild_graph_and_evaluate_all(&mut self) {
        self.dependencies.clear();
        self.dependents.clear();

        for r in 0..self.max_rows {
            for c in 0..self.max_cols {
                let trimmed = self.data[r][c].raw.trim().to_string();
                if trimmed.starts_with('=') {
                    let refs = self.extract_references(&trimmed[1..]);
                    if self.creates_cycle((r, c), &refs) {
                        self.data[r][c].computed = CellValue::Error(CellError::Circular);
                    } else {
                        self.dependencies.insert((r, c), refs.clone());
                        for dep in refs {
                            self.dependents.entry(dep).or_default().insert((r, c));
                        }
                    }
                }
            }
        }

        for r in 0..self.max_rows {
            for c in 0..self.max_cols {
                self.recompute_cell(r, c);
            }
        }
    }

    fn extract_references(&self, expr: &str) -> HashSet<(usize, usize)> {
        let mut refs = HashSet::new();
        if let Ok(tokens) = self.tokenize(expr) {
            for tok in tokens {
                if let Token::CellRef(r, c) = tok {
                    if r < self.max_rows && c < self.max_cols {
                        refs.insert((r, c));
                    }
                }
            }
        }
        refs
    }

    fn eval_expression(&self, expr: &str) -> CellValue {
        let tokens = match self.tokenize(expr) {
            Ok(t) => t,
            Err(e) => return CellValue::Error(e),
        };

        if tokens.is_empty() {
            return CellValue::Error(CellError::Syntax);
        }

        let mut idx = 0;
        let res = self.parse_additive(&tokens, &mut idx);

        if idx < tokens.len() {
            return CellValue::Error(CellError::Syntax);
        }

        res
    }

    fn parse_additive(&self, tokens: &[Token], idx: &mut usize) -> CellValue {
        let mut left = match self.parse_multiplicative(tokens, idx) {
            CellValue::Number(n) => n,
            err => return err,
        };

        while *idx < tokens.len() {
            match &tokens[*idx] {
                Token::Plus => {
                    *idx += 1;
                    let right = match self.parse_multiplicative(tokens, idx) {
                        CellValue::Number(n) => n,
                        err => return err,
                    };
                    left += right;
                }
                Token::Minus => {
                    *idx += 1;
                    let right = match self.parse_multiplicative(tokens, idx) {
                        CellValue::Number(n) => n,
                        err => return err,
                    };
                    left -= right;
                }
                _ => break,
            }
        }

        CellValue::Number(left)
    }

    fn parse_multiplicative(&self, tokens: &[Token], idx: &mut usize) -> CellValue {
        let mut left = match self.parse_unary(tokens, idx) {
            CellValue::Number(n) => n,
            err => return err,
        };

        while *idx < tokens.len() {
            match &tokens[*idx] {
                Token::Star => {
                    *idx += 1;
                    let right = match self.parse_unary(tokens, idx) {
                        CellValue::Number(n) => n,
                        err => return err,
                    };
                    left *= right;
                }
                Token::Slash => {
                    *idx += 1;
                    let right = match self.parse_unary(tokens, idx) {
                        CellValue::Number(n) => n,
                        err => return err,
                    };
                    if right == 0.0 {
                        return CellValue::Error(CellError::DivByZero);
                    }
                    left /= right;
                }
                _ => break,
            }
        }

        CellValue::Number(left)
    }

    fn parse_unary(&self, tokens: &[Token], idx: &mut usize) -> CellValue {
        if *idx >= tokens.len() {
            return CellValue::Error(CellError::Syntax);
        }

        match &tokens[*idx] {
            Token::Minus => {
                *idx += 1;
                match self.parse_unary(tokens, idx) {
                    CellValue::Number(n) => CellValue::Number(-n),
                    err => err,
                }
            }
            Token::Plus => {
                *idx += 1;
                self.parse_unary(tokens, idx)
            }
            _ => self.parse_primary(tokens, idx),
        }
    }

    fn parse_primary(&self, tokens: &[Token], idx: &mut usize) -> CellValue {
        if *idx >= tokens.len() {
            return CellValue::Error(CellError::Syntax);
        }

        match &tokens[*idx] {
            Token::Number(n) => {
                let val = *n;
                *idx += 1;
                CellValue::Number(val)
            }
            Token::CellRef(r, c) => {
                let row = *r;
                let col = *c;
                *idx += 1;

                if row >= self.max_rows || col >= self.max_cols {
                    return CellValue::Error(CellError::Ref);
                }

                match &self.data[row][col].computed {
                    CellValue::Number(n) => CellValue::Number(*n),
                    CellValue::Text(s) => {
                        if let Ok(n) = s.parse::<f64>() {
                            CellValue::Number(n)
                        } else if s.is_empty() {
                            CellValue::Number(0.0)
                        } else {
                            CellValue::Error(CellError::Value)
                        }
                    }
                    CellValue::Error(e) => CellValue::Error(e.clone()),
                }
            }
            Token::LParen => {
                *idx += 1;
                let val = self.parse_additive(tokens, idx);
                if *idx < tokens.len() && tokens[*idx] == Token::RParen {
                    *idx += 1;
                    val
                } else {
                    CellValue::Error(CellError::Syntax)
                }
            }
            _ => CellValue::Error(CellError::Syntax),
        }
    }

    fn tokenize(&self, expr: &str) -> Result<Vec<Token>, CellError> {
        let mut tokens = Vec::new();
        let chars: Vec<char> = expr.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            match chars[i] {
                ' ' | '\t' | '\r' | '\n' => i += 1,
                '+' => {
                    tokens.push(Token::Plus);
                    i += 1;
                }
                '-' => {
                    tokens.push(Token::Minus);
                    i += 1;
                }
                '*' => {
                    tokens.push(Token::Star);
                    i += 1;
                }
                '/' => {
                    tokens.push(Token::Slash);
                    i += 1;
                }
                '(' => {
                    tokens.push(Token::LParen);
                    i += 1;
                }
                ')' => {
                    tokens.push(Token::RParen);
                    i += 1;
                }
                '0'..='9' | '.' => {
                    let start = i;
                    while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                        i += 1;
                    }
                    let num_str: String = chars[start..i].iter().collect();
                    let num = num_str.parse::<f64>().map_err(|_| CellError::Syntax)?;
                    tokens.push(Token::Number(num));
                }
                'A'..='Z' | 'a'..='z' => {
                    let start_col = i;
                    while i < chars.len() && chars[i].is_ascii_alphabetic() {
                        i += 1;
                    }
                    let col_str: String = chars[start_col..i]
                        .iter()
                        .map(|c| c.to_ascii_uppercase())
                        .collect();

                    let start_row = i;
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        i += 1;
                    }
                    if start_row == i {
                        return Err(CellError::Syntax);
                    }
                    let row_str: String = chars[start_row..i].iter().collect();
                    let row_num = row_str.parse::<usize>().map_err(|_| CellError::Syntax)?;

                    if row_num == 0 {
                        return Err(CellError::Ref);
                    }

                    let col_idx = Self::letter_to_col(&col_str)?;
                    let row_idx = row_num - 1;

                    tokens.push(Token::CellRef(row_idx, col_idx));
                }
                _ => return Err(CellError::Syntax),
            }
        }

        Ok(tokens)
    }

    fn letter_to_col(letters: &str) -> Result<usize, CellError> {
        let mut col = 0;
        for c in letters.chars() {
            if !c.is_ascii_uppercase() {
                return Err(CellError::Syntax);
            }
            col = col * 26 + ((c as usize) - ('A' as usize) + 1);
        }
        if col == 0 {
            return Err(CellError::Syntax);
        }
        Ok(col - 1)
    }

    pub fn save_to_csv<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::create(path)?;
        let mut wtr = csv::WriterBuilder::new()
            .has_headers(false)
            .from_writer(file);

        let mut last_row = 0;
        for r in (0..self.max_rows).rev() {
            if self.data[r].iter().any(|c| !c.raw.is_empty()) {
                last_row = r;
                break;
            }
        }

        let mut last_col = 0;
        for row in &self.data {
            for c in (0..self.max_cols).rev() {
                if !row[c].raw.is_empty() {
                    if c > last_col {
                        last_col = c;
                    }
                    break;
                }
            }
        }

        for r in 0..=last_row {
            let row_raws: Vec<String> = self.data[r][0..=last_col]
                .iter()
                .map(|cell| cell.raw.clone())
                .collect();
            wtr.write_record(&row_raws)?;
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

#[derive(Debug, PartialEq)]
enum Token {
    Number(f64),
    CellRef(usize, usize),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
}
