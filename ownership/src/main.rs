//! Chapter 4 — Understanding Ownership.
//!
//! Covers 4.1 (what ownership is), 4.2 (references and borrowing), and 4.3 (slices):
//!
//!     cargo run -p ownership
//!     cargo test -p ownership
//!
//! Lines marked `ERROR[E0xxx]` are code the compiler rejects. They are commented out so this crate
//! builds; uncomment one at a time and read the compiler's message, or run `rustc --explain E0382`
//! in a terminal.

fn main() {
    ownership_rules();
    stack_and_heap();
    moves();
    clones();
    copy_types();
    ownership_and_functions();
    returning_ownership();
    immutable_borrows();
    mutable_borrows();
    slices();
}

fn ownership_rules() {
    println!("== Ownership rules (4.1) ==");
    println!("1. Every value has exactly one owner.");
    println!("2. There can only be one owner at a time.");
    println!("3. When the owner goes out of scope, the value is dropped.");
    println!("The compiler checks these rules; nothing runs at runtime to enforce them.");
}

fn stack_and_heap() {
    println!("\n== Stack and heap ==");

    // `s` (pointer, length, capacity) lives on the stack; its bytes live on the heap.
    let mut s = String::from("hello");
    s.push_str(", world!");
    println!("heap-backed String: {s}");

    // Fixed-size data lives entirely on the stack. Assigning it copies the bits.
    let n: i32 = 42;
    let m = n;
    println!("stack values: n = {n}, m = {m}");
}

fn moves() {
    println!("\n== Move ==");

    let s1 = String::from("hello");
    let s2 = s1; // the String is *moved*: s1 is no longer the owner

    println!("s2 owns the string: {s2}");
    // println!("{s1}"); // ERROR[E0382]: borrow of moved value: `s1`

    // The same thing happens when a value is passed to a function (see below).
}

fn clones() {
    println!("\n== Clone ==");

    let s1 = String::from("hello");
    let s2 = s1.clone(); // deep copy: two independent heap buffers

    println!("s1 = {s1}, s2 = {s2}");
    println!("Cloning allocates. Reach for it only when both values must outlive each other.");
}

fn copy_types() {
    println!("\n== Copy ==");

    let x = 5;
    let y = x; // i32 is Copy, so `x` stays valid
    println!("x = {x}, y = {y}");

    // Tuples (and arrays) of Copy types are Copy too.
    let pair = (1, 2.5, 'r');
    let copied = pair;
    println!("pair = {pair:?}, copied = {copied:?}");
    println!("String is not Copy: duplicating its heap buffer would be a move or a clone.");
}

fn takes_ownership(text: String) {
    println!("takes_ownership() now owns: {text}");
} // `text` is dropped here

fn measure(text: &str) -> usize {
    text.len()
}

fn ownership_and_functions() {
    println!("\n== Ownership and functions ==");

    let text = String::from("hello");
    takes_ownership(text);
    // println!("{text}"); // ERROR[E0382]: borrow of moved value: `text`

    // Borrowing instead of moving lets the caller keep ownership.
    let text = String::from("hello");
    let len = measure(&text); // `&String` coerces to `&str`
    println!("measure() returned {len}; text is still usable: {text}");
}

fn make_greeting() -> String {
    String::from("hello from a returned String")
}

fn pass_through(text: String) -> String {
    text
}

fn returning_ownership() {
    println!("\n== Returning ownership ==");

    let greeting = make_greeting();
    let greeting = pass_through(greeting);
    println!("{greeting}");
}

fn immutable_borrows() {
    println!("\n== References and borrowing (4.2) ==");

    let s = String::from("hello");

    // Any number of immutable borrows may coexist.
    let r1 = &s;
    let r2 = &s;
    println!("many immutable borrows: {r1} and {r2}");

    // `&String` works, but `&str` accepts more callers.
    let by_ref: &String = &s;
    println!("borrowed as &String, length = {}", by_ref.len());

    // A borrow ends at its last use (non-lexical lifetimes), not at the end of the block.
    // That is why the mutation below is allowed even though `r` existed earlier.
    let mut s = String::from("hello");
    let r = &s;
    println!("before mutation: {r}");
    s.push_str(", world");
    println!("after mutation:  {s}");

    // Returning a reference to a local would need a lifetime the compiler cannot satisfy:
    // fn dangle() -> &String { &String::from("x") } // ERROR[E0106]
}

fn append_world(text: &mut String) {
    text.push_str(", world");
}

fn mutable_borrows() {
    println!("\n== Mutable borrows ==");

    let mut s = String::from("hello");
    append_world(&mut s);
    println!("after append_world: {s}");

    // Exactly one mutable borrow is allowed at a time.
    let first = &mut s;
    // let second = &mut s; // ERROR[E0499]: cannot borrow `s` as mutable more than once
    first.push('!');
    println!("after push: {s}");

    // A mutable borrow also excludes immutable borrows while both are live.
    // let immutable = &s;
    // let mutable = &mut s; // ERROR[E0502]: cannot borrow `s` as mutable because it is also borrowed as immutable
    // println!("{immutable} {mutable}");
}

/// Returns the first whitespace-delimited word of `text`.
///
/// The returned `&str` points into `text`, so it borrows from the argument. The compiler will not
/// let the caller drop or mutate `text` while the returned slice is alive.
fn first_word(text: &str) -> &str {
    for (index, byte) in text.as_bytes().iter().enumerate() {
        if *byte == b' ' {
            return &text[..index];
        }
    }
    text
}

fn sum(values: &[i32]) -> i32 {
    values.iter().sum()
}

fn double_all(values: &mut [i32]) {
    for value in values {
        *value *= 2;
    }
}

fn slices() {
    println!("\n== Slices (4.3) ==");

    let sentence = String::from("hello world");
    let hello = &sentence[..5];
    let world = &sentence[6..];
    println!("&sentence[..5] = {hello:?}, &sentence[6..] = {world:?}");
    println!("first_word(sentence) = {:?}", first_word(&sentence));

    // Array slices: a borrowed view of part of a collection, without copying it.
    let numbers = [1, 2, 3, 4];
    println!("sum(&numbers) = {}", sum(&numbers));

    let mut numbers = [1, 2, 3, 4];
    double_all(&mut numbers);
    println!("double_all(&mut numbers) -> {numbers:?}");

    // `&String` coerces to `&str`, so one function serves both.
    let owned = String::from("borrowed");
    println!("first_word(&owned) = {:?}", first_word(&owned));

    // Slicing is byte-based. Cutting through a multi-byte character panics at runtime:
    let accented = "héllo";
    // let broken = &accented[..2]; // panics: byte index 2 is not a char boundary
    println!("first_word(accented) = {:?}", first_word(accented));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_word_finds_the_first_word() {
        assert_eq!(first_word("hello world"), "hello");
    }

    #[test]
    fn first_word_returns_the_whole_string_when_there_is_no_space() {
        assert_eq!(first_word("hello"), "hello");
    }

    #[test]
    fn first_word_handles_empty_input() {
        assert_eq!(first_word(""), "");
    }

    #[test]
    fn first_word_returns_an_empty_slice_when_the_string_starts_with_a_space() {
        assert_eq!(first_word(" hello"), "");
    }

    #[test]
    fn first_word_does_not_split_multibyte_characters() {
        assert_eq!(first_word("héllo world"), "héllo");
    }

    #[test]
    fn sum_adds_every_element() {
        assert_eq!(sum(&[1, 2, 3, 4]), 10);
    }

    #[test]
    fn sum_of_an_empty_slice_is_zero() {
        assert_eq!(sum(&[]), 0);
    }

    #[test]
    fn double_all_mutates_in_place() {
        let mut values = [1, 2, 3];
        double_all(&mut values);
        assert_eq!(values, [2, 4, 6]);
    }

    #[test]
    fn clone_produces_an_independent_value() {
        let mut original = String::from("hello");
        let copy = original.clone();
        original.push_str(" world");
        assert_eq!(copy, "hello");
    }

    #[test]
    fn copy_leaves_the_source_usable() {
        let x = 5;
        let y = x;
        assert_eq!((x, y), (5, 5));
    }
}
