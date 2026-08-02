use rSheet::domain::Spreadsheet;

fn main() {
    println!("Testing circular dependency detection with functions...\n");

    let mut sheet = Spreadsheet::new(100, 26);

    // First, set C2 to a formula
    sheet.set_cell(1, 2, "=C1+1".to_string());
    println!("Set C2 = =C1+1 (C1 is empty, so C2 should be 1)");
    println!("C2 = {}", sheet.get_cell(1, 2).unwrap().display_text());

    // Now try to set C1 to depend on C2 - this should create a cycle
    println!("\nNow trying to set C1 = =C2+1 (should create circular dependency)...");
    sheet.set_cell(0, 2, "=C2+1".to_string());
    println!("C1 = {}", sheet.get_cell(0, 2).unwrap().display_text());
    println!("C2 = {}", sheet.get_cell(1, 2).unwrap().display_text());

    // Test with functions
    println!("\n\nTesting with functions:");
    sheet.set_cell(0, 3, "=sum(D2,1)".to_string());
    println!("Set D1 = =sum(D2,1)");
    println!("D1 = {}", sheet.get_cell(0, 3).unwrap().display_text());

    println!("\nNow trying to set D2 = =pow(D1,2) (should create circular dependency)...");
    sheet.set_cell(1, 3, "=pow(D1,2)".to_string());
    println!("D2 = {}", sheet.get_cell(1, 3).unwrap().display_text());
    println!("D1 = {}", sheet.get_cell(0, 3).unwrap().display_text());

    println!("\n✓ Circular dependency detection test complete!");
}
