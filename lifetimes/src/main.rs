//! Chapter 10.3 — Validating References with Lifetimes.
//!
//!     cargo run -p lifetimes
//!     cargo test -p lifetimes
//!
//! A lifetime is a name for "how long this reference is valid". Annotations do not change how long
//! anything lives; they describe relationships so the borrow checker can verify callers.

fn main() {
    longest_demo();
    scope_demo();
    struct_with_reference();
    elision_rules();
    static_lifetime();
}

/// Returns the longer of two string slices.
///
/// `'a` promises that the returned reference is valid for no longer than *both* inputs. In practice
/// the compiler uses the shorter of the two lifetimes: the result may not outlive either argument.
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() >= y.len() { x } else { y }
}

fn longest_demo() {
    println!("== Lifetime parameters ==");

    let first = String::from("long string is long");
    let second = String::from("xyz");
    println!("longest = {:?}", longest(&first, &second));

    // Literals are `&'static str`, which outlives everything here.
    println!("literals: {:?}", longest("abc", "de"));
}

fn scope_demo() {
    println!("\n== The result cannot outlive its inputs ==");

    let long_lived = String::from("long");
    let result;
    {
        let short_lived = String::from("loooooong");
        result = longest(&long_lived, &short_lived);
        println!("inside the block: {result}");
    }

    // `short_lived` is dropped at the end of the block, so `result` would be dangling:
    // println!("after the block: {result}"); // ERROR[E0597]: `short_lived` does not live long enough
}

/// A struct that borrows part of a string instead of copying it.
///
/// The `'a` says: an `Excerpt` cannot outlive the text it points into.
#[derive(Debug)]
struct Excerpt<'a> {
    part: &'a str,
}

impl<'a> Excerpt<'a> {
    fn new(part: &'a str) -> Self {
        Self { part }
    }

    /// Elision rule: because the method takes `&self`, the returned reference gets `self`'s
    /// lifetime — not `announcement`'s. So the caller may drop `announcement` immediately.
    fn announce_and_return(&self, announcement: &str) -> &str {
        println!("Attention: {announcement}");
        self.part
    }
}

fn struct_with_reference() {
    println!("\n== Structs that hold references ==");

    let text = String::from("Call me Ishmael. Some years ago...");
    let excerpt = Excerpt::new(&text[..16]);

    println!("{:?}", excerpt.announce_and_return("here is an excerpt"));

    // While `excerpt` is alive it borrows `text` immutably, so this would not compile:
    // text.push('!'); // ERROR[E0502]: cannot borrow `text` as mutable because it is also borrowed as immutable
}

/// One input lifetime, so the output gets that lifetime by elision.
fn first_word(s: &str) -> &str {
    s.split(' ').next().unwrap_or(s)
}

fn elision_rules() {
    println!("\n== Lifetime elision ==");

    let sentence = String::from("hello world");
    println!("first_word = {:?}", first_word(&sentence));

    // The three rules:
    // 1. Each elided input reference gets its own lifetime parameter.
    // 2. Exactly one input lifetime -> the output gets that lifetime.
    // 3. Multiple inputs, but one is `&self`/`&mut self` -> the output gets `self`'s lifetime.

    // Without elision, `first_word` would be written with an explicit lifetime. Clippy would
    // normally elide this, which is exactly the point: the annotation adds no information here.
    #[allow(clippy::needless_lifetimes)]
    fn first_word_explicit<'a>(s: &'a str) -> &'a str {
        s.split(' ').next().unwrap_or(s)
    }
    println!("explicit = {:?}", first_word_explicit(&sentence));
}

fn static_lifetime() {
    println!("\n== The 'static lifetime ==");

    let literal: &'static str = "I am valid for the whole program";
    println!("{literal}");
    println!(
        "'static is a bound, not a requirement: most references are shorter and that is fine."
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn longest_picks_the_longer_input() {
        assert_eq!(longest("long string", "short"), "long string");
        assert_eq!(longest("short", "long string"), "long string");
    }

    #[test]
    fn longest_returns_the_first_on_a_tie() {
        assert_eq!(longest("abc", "xyz"), "abc");
    }

    #[test]
    fn excerpt_returns_its_own_slice() {
        let text = String::from("Call me Ishmael.");
        let excerpt = Excerpt::new(&text[..16]);
        let announcement = String::from("temporary");

        let part = excerpt.announce_and_return(&announcement);
        drop(announcement); // proves the result does not borrow `announcement`
        assert_eq!(part, "Call me Ishmael.");
    }

    #[test]
    fn first_word_stops_at_the_first_space() {
        assert_eq!(first_word("hello world"), "hello");
        assert_eq!(first_word("hello"), "hello");
        assert_eq!(first_word(""), "");
    }

    #[test]
    fn static_strings_are_always_valid() {
        let literal: &'static str = "static";
        assert_eq!(literal.len(), 6);
    }
}
