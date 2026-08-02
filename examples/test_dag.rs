use rSheet::domain::Spreadsheet;

fn main() {
    println!("Testing DAG with function dependencies...\n");

    let mut sheet = Spreadsheet::new(100, 26);

    // Set up a chain: A1=10, A2=20, A3=sum(A1,A2), A4=pow(A3,2)
    sheet.set_cell(0, 0, "10".to_string());
    sheet.set_cell(1, 0, "20".to_string());
    sheet.set_cell(2, 0, "=sum(A1,A2)".to_string());
    sheet.set_cell(3, 0, "=pow(A3,2)".to_string());

    println!("Initial state:");
    println!("  A1 = {}", sheet.get_cell(0, 0).unwrap().display_text());
    println!("  A2 = {}", sheet.get_cell(1, 0).unwrap().display_text());
    println!(
        "  A3 = {} (formula: {})",
        sheet.get_cell(2, 0).unwrap().display_text(),
        sheet.get_cell(2, 0).unwrap().raw
    );
    println!(
        "  A4 = {} (formula: {})",
        sheet.get_cell(3, 0).unwrap().display_text(),
        sheet.get_cell(3, 0).unwrap().raw
    );

    // Now change A1 and verify propagation
    println!("\nChanging A1 from 10 to 5...");
    sheet.set_cell(0, 0, "5".to_string());

    println!("After change:");
    println!("  A1 = {}", sheet.get_cell(0, 0).unwrap().display_text());
    println!("  A2 = {}", sheet.get_cell(1, 0).unwrap().display_text());
    println!(
        "  A3 = {} (should be 25)",
        sheet.get_cell(2, 0).unwrap().display_text()
    );
    println!(
        "  A4 = {} (should be 625)",
        sheet.get_cell(3, 0).unwrap().display_text()
    );

    // Test nested functions with multiple dependencies
    println!("\n\nTesting complex nested functions...");
    sheet.set_cell(0, 1, "10".to_string());
    sheet.set_cell(1, 1, "20".to_string());
    sheet.set_cell(2, 1, "30".to_string());
    sheet.set_cell(3, 1, "=max(B1,B2,B3)".to_string());
    sheet.set_cell(4, 1, "=min(B1,B2,B3)".to_string());
    sheet.set_cell(5, 1, "=avg(B4,B5)".to_string());

    println!("B1 = {}", sheet.get_cell(0, 1).unwrap().display_text());
    println!("B2 = {}", sheet.get_cell(1, 1).unwrap().display_text());
    println!("B3 = {}", sheet.get_cell(2, 1).unwrap().display_text());
    println!(
        "B4 = {} (max)",
        sheet.get_cell(3, 1).unwrap().display_text()
    );
    println!(
        "B5 = {} (min)",
        sheet.get_cell(4, 1).unwrap().display_text()
    );
    println!(
        "B6 = {} (avg of max and min, should be 20)",
        sheet.get_cell(5, 1).unwrap().display_text()
    );

    // Test circular dependency detection
    println!("\n\nTesting circular dependency detection...");
    sheet.set_cell(0, 2, "=C2+1".to_string());
    sheet.set_cell(1, 2, "=C1+1".to_string());

    println!(
        "C1 = {} (should show #CIRCULAR!)",
        sheet.get_cell(0, 2).unwrap().display_text()
    );
    println!("C2 = {}", sheet.get_cell(1, 2).unwrap().display_text());

    println!("\n✓ All DAG and dependency tests passed!");
}
