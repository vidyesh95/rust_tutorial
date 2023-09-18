/*
* Rust has a number of built-in operators and they mostly work the same way as in other languages.
* Rust has four main categories of operators: arithmetic, comparison, logical, and bitwise.
* Rust also has the following bitwise operators that only work on integers:
* Bitwise AND: &
* Bitwise OR: |
* Bitwise XOR: ^
* Bitwise NOT: !
* Bitwise left shift: <<
* Bitwise right shift: >>
*/
fn main() {
    println!("Arithmetic operators");

    // Addition
    let sum = 5 + 10;
    println!("The value of sum is: {sum}");

    // Subtraction
    let difference = 95.5 - 4.3;
    println!("The value of difference is: {difference}");

    // Multiplication
    let product = 4 * 30;
    println!("The value of product is: {product}");

    // Division
    let quotient = 56.7 / 32.2;
    println!("The value of quotient is: {quotient}");

    // Remainder
    let remainder = 43 % 5;
    println!("The value of remainder is: {remainder}");

    println!("Comparison operators");

    // Equal to
    let is_equal = 5 == 5;
    println!("The value of is_equal is: {is_equal}");

    // Not equal to
    let is_not_equal = 5 != 5;
    println!("The value of is_not_equal is: {is_not_equal}");

    // Greater than
    let is_greater_than = 5 > 5;
    println!("The value of is_greater_than is: {is_greater_than}");

    // Less than
    let is_less_than = 5 < 5;
    println!("The value of is_less_than is: {is_less_than}");

    // Greater than or equal to
    let is_greater_than_or_equal_to = 5 >= 5;
    println!("The value of is_greater_than_or_equal_to is: {is_greater_than_or_equal_to}");

    // Less than or equal to
    let is_less_than_or_equal_to = 5 <= 5;
    println!("The value of is_less_than_or_equal_to is: {is_less_than_or_equal_to}");

    println!("Logical operators");

    // Logical AND
    let logical_and = true && false;
    println!("The value of logical_and is: {logical_and}");

    // Logical OR
    let logical_or = true || false;
    println!("The value of logical_or is: {logical_or}");

    // Logical NOT
    let logical_not = !true;
    println!("The value of logical_not is: {logical_not}");

    println!("Bitwise operators");

    // Bitwise AND
    let bitwise_and = 0b1111 & 0b0000;
    println!("The value of bitwise_and is: {bitwise_and}");

    // Bitwise OR
    let bitwise_or = 0b1111 | 0b0000;
    println!("The value of bitwise_or is: {bitwise_or}");

    // Bitwise XOR
    let bitwise_xor = 0b1111 ^ 0b0000;
    println!("The value of bitwise_xor is: {bitwise_xor}");

    // Bitwise NOT
    let bitwise_not = !0b1111;
    println!("The value of bitwise_not is: {bitwise_not}");

    // Bitwise left shift
    let bitwise_left_shift = 0b0001 << 1;
    println!("The value of bitwise_left_shift is: {bitwise_left_shift}");

    // Bitwise right shift
    let bitwise_right_shift = 0b0001 >> 1;
    println!("The value of bitwise_right_shift is: {bitwise_right_shift}");

    println!("Compound assignment operators");

    // +=
    let mut x = 5;
    println!("The value of x is: {x}");
    x += 1;
    println!("The value of x is: {x}");

    // -=
    let mut x = 5;
    println!("The value of x is: {x}");
    x -= 1;
    println!("The value of x is: {x}");

    // *=
    let mut x = 5;
    println!("The value of x is: {x}");
    x *= 1;
    println!("The value of x is: {x}");

    // /=
    let mut x = 5;
    println!("The value of x is: {x}");
    x /= 1;
    println!("The value of x is: {x}");

    // %=
    let mut x = 5;
    println!("The value of x is: {x}");
    x %= 1;
    println!("The value of x is: {x}");

    // &=
    let mut x = 0b1111;
    println!("The value of x is: {x}");
    x &= 0b0000;
    println!("The value of x is: {x}");

    // |=
    let mut x = 0b1111;
    println!("The value of x is: {x}");
    x |= 0b0000;
    println!("The value of x is: {x}");

    // ^=
    let mut x = 0b1111;
    println!("The value of x is: {x}");
    x ^= 0b0000;
    println!("The value of x is: {x}");

    // <<=
    let mut x = 0b0001;
    println!("The value of x is: {x}");
    x <<= 1;
    println!("The value of x is: {x}");

    // >>=
    let mut x = 0b0001;
    println!("The value of x is: {x}");
    x >>= 1;
    println!("The value of x is: {x}");

    println!("Precedence and associativity");

    // Precedence
    let x = 5 + 5 * 5;
    println!("The value of x is: {x}");

    // Associativity
    let x = 5 + 5 + 5;
    println!("The value of x is: {x}");

    let x = 5 + (5 + 5);
    println!("The value of x is: {x}");
}
