//! Domain layer: types and business rules.
//!
//! The `User` fields are private on purpose. Callers must go through `User::new`, so the invariant
//! "a user always has a non-empty, normalized name" holds everywhere in the program.

use std::fmt;

use crate::internal::normalize_name;

/// A user of the service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    id: u64,
    name: String,
}

/// Errors the domain layer can return.
#[derive(Debug, PartialEq, Eq)]
pub enum DomainError {
    /// The supplied name was empty after trimming.
    EmptyName,
}

impl fmt::Display for DomainError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyName => write!(formatter, "name must not be empty"),
        }
    }
}

impl std::error::Error for DomainError {}

impl User {
    /// Creates a user, trimming whitespace and lower-casing the name.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::EmptyName`] when the normalized name is empty.
    pub fn new(id: u64, raw_name: &str) -> Result<Self, DomainError> {
        let name = normalize_name(raw_name);
        if name.is_empty() {
            return Err(DomainError::EmptyName);
        }
        Ok(Self { id, name })
    }

    /// The user's identifier.
    #[must_use]
    pub fn id(&self) -> u64 {
        self.id
    }

    /// The normalized name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_and_lowercases_the_name() {
        let user = User::new(1, "  Ada Lovelace  ").expect("valid name");
        assert_eq!(user.name(), "ada lovelace");
    }

    #[test]
    fn rejects_a_blank_name() {
        let error = User::new(1, "   ").expect_err("blank names are rejected");
        assert_eq!(error, DomainError::EmptyName);
    }
}
