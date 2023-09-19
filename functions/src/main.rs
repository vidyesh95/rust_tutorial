/*
* Rust code uses snake case as the conventional style for function and variable names.
* In snake case, all letters are lowercase and underscores separate words.
*/

fn main() {
    println!("Hello, world!");
    another_function(5);

    print_labeled_measurement(5, 'h');

    let y = {
        let x = 3;
        x + 1 // Expressions do not include ending semicolons.
              // If you add a semicolon to the end of an expression,
              // you turn it into a statement, which will then not return a value.
    };
    println!("The value of y is: {y}");

    let sev = seven();
    println!("The value of sev is: {sev}");

    let plu = plus_one(5);
    println!("The value of plu is: {plu}");
}

/**
 * @param x: i32 - A signed 32-bit integer.
 */
fn another_function(x: i32) {
    println!("Another function.");
    println!("The value of x is: {x}");
}

/**
 * @param value: i32 - A signed 32-bit integer.
 * @param unit_label: char - A character.
 */
fn print_labeled_measurement(value: i32, unit_label: char) {
    println!("The measurement is: {value}{unit_label}");
}

// Functions with Return Values
fn seven() -> i32 {
    7
}

fn plus_one(x: i32) -> i32 {
    x + 1
}
