/**
* Rust is a statically typed language, which means that it must know the types of all variables at
* compile time.
* The compiler can usually infer what type we want to use based on the value and how we use it.
* In cases when many types are possible, such as when converting a String to a number, we must
* add a type annotation.
* Rust has a number of scalar types, which represent a single value.
* Rust has four primary scalar types: integers, floating-point numbers, Booleans, and characters.
* Signed integers can store positive and negative numbers.
* Signed integer types are i8, i16, i32, i64, i128 and isize.
* Unsigned integers can store only positive numbers.
* Unsigned integer types are u8, u16, u32, u64, u128 and usize.
* The isize and usize types depend on the kind of computer your program is running on: 64 bits if
* you’re on a 64-bit architecture and 32 bits if you’re on a 32-bit architecture.
* Rust’s floating-point types are f32 and f64, which are 32 bits and 64 bits in size, respectively.
* The Boolean type in Rust has two possible values: true and false.
* Rust’s char type is four bytes in size and represents a Unicode Scalar Value, which means it can
* represent a lot more than just ASCII.
* Rust has two primitive compound types: tuples and arrays.
* A tuple is a general way of grouping together a number of values with a variety of types into
* one compound type.
* Tuples have a fixed length: once declared, they cannot grow or shrink in size.
* We create a tuple by writing a comma-separated list of values inside parentheses.
* Each position in the tuple has a type, and the types of the different values in the tuple don’t
* have to be the same.
*/
fn main() {
    println!("Signed integers!");
    let signed_integer8: i8 = 127;
    println!("The value of signed_integer8 is: {signed_integer8}");

    let signed_integer16: i16 = 32_767;
    println!("The value of signed_integer16 is: {signed_integer16}");

    let signed_integer32: i32 = 2_147_483_647;
    println!("The value of signed_integer32 is: {signed_integer32}");

    let signed_integer64: i64 = 9_223_372_036_854_775_807;
    println!("The value of signed_integer64 is: {signed_integer64}");

    let signed_integer128: i128 = 170_141_183_460_469_231_731_687_303_715_884_105_727;
    println!("The value of signed_integer128 is: {signed_integer128}");

    let signed_integer_size: isize = 9_223_372_036_854_775_807;
    println!("The value of signed_integer_size is: {signed_integer_size}");

    println!("Unsigned integers!");
    let unsigned_integer8: u8 = 255;
    println!("The value of unsigned_integer8 is: {unsigned_integer8}");

    let unsigned_integer16: u16 = 65_535;
    println!("The value of unsigned_integer16 is: {unsigned_integer16}");

    let unsigned_integer32: u32 = 4_294_967_295;
    println!("The value of unsigned_integer32 is: {unsigned_integer32}");

    let unsigned_integer64: u64 = 18_446_744_073_709_551_615;
    println!("The value of unsigned_integer64 is: {unsigned_integer64}");

    let unsigned_integer128: u128 = 340_282_366_920_938_463_463_374_607_431_768_211_455;
    println!("The value of unsigned_integer128 is: {unsigned_integer128}");

    let unsigned_integer_size: usize = 18_446_744_073_709_551_615;
    println!("The value of unsigned_integer_size is: {unsigned_integer_size}");

    println!("Floating-point types");
    let volume32: f32 = 1_000_000.8;
    println!("The value of volume32 is: {volume32}");

    let volume64: f64 = 1_000_000.3459;
    println!("The value of volume64 is: {volume64}");

    println!("Boolean type");
    let is_active: bool = false;
    println!("The value of is_active is: {is_active}");

    println!("Character type");
    let character : char= 'z';
    println!("The value of character is: {character}");

    println!("Tuples!");
    let tup: (i32, f64, u8) = (500, 6.4, 1);
    let (_x, y, _z) = tup; // Destructuring

    println!("The value of y is: {y}");

    let a: (i32, f64, u8) = (500, 6.4, 1);

    let five_hundred = a.0;
    println!("The value of five_hundred is: {five_hundred}");

    let six_point_four = a.1;
    println!("The value of six_point_four is: {six_point_four}");

    let one = a.2;
    println!("The value of one is: {one}");

    println!("Arrays!");
    let a = [1, 2, 3, 4, 5];
    let _a: [i32; 5] = [1, 2, 3, 4, 5];
    println!("The value of a is: {:?}",a);
    println!("Please enter an array index.");
    let mut index = String::new();
    std::io::stdin()
        .read_line(&mut index)
        .expect("Failed to read line");
    let index: usize = index
        .trim()
        .parse()
        .expect("Index entered was not a number");
    let element = a[index];
    println!(
        "The value of the element at index {} is: {}",
        index, element
    );

    let months:[&str;12] = ["January", "February", "March", "April", "May", "June", "July",
        "August", "September", "October", "November", "December"];
    println!("The value of months is: {:?}",months);

    let b = [3; 5]; // [3, 3, 3, 3, 3]
    println!("The value of b is:{:?}",b);
}
