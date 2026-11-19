//! Private module: `mod internal;` in `lib.rs` makes this invisible outside the crate.
//!
//! `pub(crate)` means "visible anywhere in this crate, but not to dependants".

/// Trims surrounding whitespace and lower-cases the name.
pub(crate) fn normalize_name(raw: &str) -> String {
    raw.trim().to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_spacing_and_case() {
        assert_eq!(normalize_name("  Ada Lovelace "), "ada lovelace");
    }
}
