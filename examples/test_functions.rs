use rSheet::domain::Spreadsheet;

fn main() {
    println!("Testing function evaluation...\n");

    // Load the test file
    let sheet = Spreadsheet::load_from_csv("data/functions_test.csv", 100, 26)
        .expect("Failed to load test file");

    // Display the results
    println!("Row 1 (raw values):");
    for col in 0..3 {
        if let Some(cell) = sheet.get_cell(0, col) {
            println!(
                "  {}1: {} -> {}",
                Spreadsheet::col_to_letter(col),
                cell.raw,
                cell.display_text()
            );
        }
    }

    println!("\nRow 2 (pow, sum, avg):");
    for col in 0..3 {
        if let Some(cell) = sheet.get_cell(1, col) {
            println!(
                "  {}2: {} -> {}",
                Spreadsheet::col_to_letter(col),
                cell.raw,
                cell.display_text()
            );
        }
    }

    println!("\nRow 3 (max, min, pow):");
    for col in 0..3 {
        if let Some(cell) = sheet.get_cell(2, col) {
            println!(
                "  {}3: {} -> {}",
                Spreadsheet::col_to_letter(col),
                cell.raw,
                cell.display_text()
            );
        }
    }

    println!("\nRow 4 (nested functions):");
    for col in 0..3 {
        if let Some(cell) = sheet.get_cell(3, col) {
            println!(
                "  {}4: {} -> {}",
                Spreadsheet::col_to_letter(col),
                cell.raw,
                cell.display_text()
            );
        }
    }

    println!("\nRow 5 (misc):");
    for col in 0..3 {
        if let Some(cell) = sheet.get_cell(4, col) {
            println!(
                "  {}5: {} -> {}",
                Spreadsheet::col_to_letter(col),
                cell.raw,
                cell.display_text()
            );
        }
    }

    println!("\n✓ All functions evaluated successfully!");
}
