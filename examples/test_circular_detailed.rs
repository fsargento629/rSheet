use rSheet::domain::Spreadsheet;

fn main() {
    println!("Comprehensive circular dependency test...\n");

    let mut sheet = Spreadsheet::new(100, 26);

    // Test 1: Simple direct cycle
    println!("Test 1: Direct cycle A1->A1");
    sheet.set_cell(0, 0, "=A1+1".to_string());
    println!(
        "A1 = {} (raw: {})",
        sheet.get_cell(0, 0).unwrap().display_text(),
        sheet.get_cell(0, 0).unwrap().raw
    );

    // Test 2: Two-cell cycle
    println!("\nTest 2: Two-cell cycle");
    sheet.set_cell(0, 1, "10".to_string());
    sheet.set_cell(1, 1, "=B1+1".to_string());
    println!(
        "B1 = {} (raw: {})",
        sheet.get_cell(0, 1).unwrap().display_text(),
        sheet.get_cell(0, 1).unwrap().raw
    );
    println!(
        "B2 = {} (raw: {})",
        sheet.get_cell(1, 1).unwrap().display_text(),
        sheet.get_cell(1, 1).unwrap().raw
    );

    println!("\nNow trying to create cycle: B1 depends on B2...");
    sheet.set_cell(0, 1, "=B2+1".to_string());
    println!(
        "B1 = {} (raw: {}) - should be #CIRCULAR!",
        sheet.get_cell(0, 1).unwrap().display_text(),
        sheet.get_cell(0, 1).unwrap().raw
    );
    println!(
        "B2 = {} (raw: {})",
        sheet.get_cell(1, 1).unwrap().display_text(),
        sheet.get_cell(1, 1).unwrap().raw
    );

    // Test 3: Three-cell cycle with functions
    println!("\nTest 3: Three-cell cycle with functions");
    sheet.set_cell(0, 2, "5".to_string());
    sheet.set_cell(1, 2, "=pow(C1,2)".to_string());
    sheet.set_cell(2, 2, "=sum(C2,10)".to_string());
    println!(
        "C1 = {} (raw: {})",
        sheet.get_cell(0, 2).unwrap().display_text(),
        sheet.get_cell(0, 2).unwrap().raw
    );
    println!(
        "C2 = {} (raw: {})",
        sheet.get_cell(1, 2).unwrap().display_text(),
        sheet.get_cell(1, 2).unwrap().raw
    );
    println!(
        "C3 = {} (raw: {})",
        sheet.get_cell(2, 2).unwrap().display_text(),
        sheet.get_cell(2, 2).unwrap().raw
    );

    println!("\nNow trying to create cycle: C1 depends on C3...");
    sheet.set_cell(0, 2, "=avg(C3,5)".to_string());
    println!(
        "C1 = {} (raw: {}) - should be #CIRCULAR!",
        sheet.get_cell(0, 2).unwrap().display_text(),
        sheet.get_cell(0, 2).unwrap().raw
    );

    println!("\n✓ Circular dependency tests complete!");
}
