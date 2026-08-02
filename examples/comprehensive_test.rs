use rSheet::domain::Spreadsheet;

fn main() {
    println!("=== Comprehensive Function Integration Test ===\n");

    let mut sheet = Spreadsheet::new(100, 26);
    let mut test_count = 0;
    let mut passed = 0;

    // Test 1: Basic arithmetic still works
    test_count += 1;
    sheet.set_cell(0, 0, "=2+3*4".to_string());
    let result = sheet.get_cell(0, 0).unwrap().display_text();
    if result == "14" {
        println!("✓ Test 1: Basic arithmetic (2+3*4) = {}", result);
        passed += 1;
    } else {
        println!("✗ Test 1: Expected 14, got {}", result);
    }

    // Test 2: POW function
    test_count += 1;
    sheet.set_cell(0, 1, "=pow(2,8)".to_string());
    let result = sheet.get_cell(0, 1).unwrap().display_text();
    if result == "256" {
        println!("✓ Test 2: POW function (2^8) = {}", result);
        passed += 1;
    } else {
        println!("✗ Test 2: Expected 256, got {}", result);
    }

    // Test 3: SUM with cell references
    test_count += 1;
    sheet.set_cell(0, 2, "10".to_string());
    sheet.set_cell(1, 2, "20".to_string());
    sheet.set_cell(2, 2, "30".to_string());
    sheet.set_cell(3, 2, "=sum(C1,C2,C3)".to_string());
    let result = sheet.get_cell(3, 2).unwrap().display_text();
    if result == "60" {
        println!("✓ Test 3: SUM with cell refs = {}", result);
        passed += 1;
    } else {
        println!("✗ Test 3: Expected 60, got {}", result);
    }

    // Test 4: AVG function
    test_count += 1;
    sheet.set_cell(0, 3, "=avg(10,20,30,40)".to_string());
    let result = sheet.get_cell(0, 3).unwrap().display_text();
    if result == "25" {
        println!("✓ Test 4: AVG function = {}", result);
        passed += 1;
    } else {
        println!("✗ Test 4: Expected 25, got {}", result);
    }

    // Test 5: MAX function
    test_count += 1;
    sheet.set_cell(0, 4, "=max(5,15,3,12,8)".to_string());
    let result = sheet.get_cell(0, 4).unwrap().display_text();
    if result == "15" {
        println!("✓ Test 5: MAX function = {}", result);
        passed += 1;
    } else {
        println!("✗ Test 5: Expected 15, got {}", result);
    }

    // Test 6: MIN function
    test_count += 1;
    sheet.set_cell(0, 5, "=min(5,15,3,12,8)".to_string());
    let result = sheet.get_cell(0, 5).unwrap().display_text();
    if result == "3" {
        println!("✓ Test 6: MIN function = {}", result);
        passed += 1;
    } else {
        println!("✗ Test 6: Expected 3, got {}", result);
    }

    // Test 7: Nested functions
    test_count += 1;
    sheet.set_cell(0, 6, "=sum(pow(2,2),pow(3,2),pow(4,2))".to_string());
    let result = sheet.get_cell(0, 6).unwrap().display_text();
    if result == "29" {
        println!("✓ Test 7: Nested functions (4+9+16) = {}", result);
        passed += 1;
    } else {
        println!("✗ Test 7: Expected 29, got {}", result);
    }

    // Test 8: Functions with arithmetic
    test_count += 1;
    sheet.set_cell(0, 7, "=pow(2,3)+sum(1,2,3)".to_string());
    let result = sheet.get_cell(0, 7).unwrap().display_text();
    if result == "14" {
        println!("✓ Test 8: Function + arithmetic (8+6) = {}", result);
        passed += 1;
    } else {
        println!("✗ Test 8: Expected 14, got {}", result);
    }

    // Test 9: DAG propagation through functions
    test_count += 1;
    sheet.set_cell(0, 8, "5".to_string());
    sheet.set_cell(1, 8, "=pow(I1,2)".to_string());
    sheet.set_cell(2, 8, "=sum(I2,10)".to_string());
    let result1 = sheet.get_cell(2, 8).unwrap().display_text();
    sheet.set_cell(0, 8, "3".to_string()); // Change I1
    let result2 = sheet.get_cell(2, 8).unwrap().display_text();
    if result1 == "35" && result2 == "19" {
        println!(
            "✓ Test 9: DAG propagation (35→19) = {} → {}",
            result1, result2
        );
        passed += 1;
    } else {
        println!("✗ Test 9: Expected 35→19, got {}→{}", result1, result2);
    }

    // Test 10: Complex nested with cell refs
    test_count += 1;
    sheet.set_cell(0, 9, "10".to_string());
    sheet.set_cell(1, 9, "20".to_string());
    sheet.set_cell(2, 9, "=avg(max(J1,J2),min(J1,J2))".to_string());
    let result = sheet.get_cell(2, 9).unwrap().display_text();
    if result == "15" {
        println!("✓ Test 10: Complex nested avg(max,min) = {}", result);
        passed += 1;
    } else {
        println!("✗ Test 10: Expected 15, got {}", result);
    }

    // Test 11: Function with parenthesized expressions
    test_count += 1;
    sheet.set_cell(0, 10, "=pow((1+2),3)".to_string());
    let result = sheet.get_cell(0, 10).unwrap().display_text();
    if result == "27" {
        println!("✓ Test 11: Function with parens pow((1+2),3) = {}", result);
        passed += 1;
    } else {
        println!("✗ Test 11: Expected 27, got {}", result);
    }

    // Test 12: Multiple levels of nesting
    test_count += 1;
    sheet.set_cell(0, 11, "=sum(max(1,2),avg(3,4,5),min(6,7))".to_string());
    let result = sheet.get_cell(0, 11).unwrap().display_text();
    if result == "12" {
        println!("✓ Test 12: Multi-level nesting (2+4+6) = {}", result);
        passed += 1;
    } else {
        println!("✗ Test 12: Expected 12, got {}", result);
    }

    println!("\n=== Test Summary ===");
    println!("Passed: {}/{} tests", passed, test_count);

    if passed == test_count {
        println!("\n🎉 All tests passed! Function support is working correctly.");
    } else {
        println!("\n⚠️  Some tests failed. Please review the implementation.");
    }
}
