//! # Chapter 5: Structs, Methods, Associated Functions & The Builder Pattern
//!
//! ## Key Backend Insights:
//! - **Domain Modeling**: Structs are the primary vehicle for modeling API entities,
//!   database records, request payloads, and configuration objects.
//! - **Newtype Pattern**: Tuple structs like `UserId(u64)` prevent accidental ID mix-ups
//!   (e.g., passing a `PostId` where a `UserId` is expected).
//! - **The Builder Pattern**: Industry-standard pattern for constructing complex database pools,
//!   HTTP clients, and servers with optional parameters and validation.

// --- 1. Basic Structs & Newtype Pattern ---

/// Tuple struct providing strong type safety (Newtype Pattern).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UserId(pub u64);

/// Domain entity representing a user in our system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub email: String,
    pub is_active: bool,
    pub sign_in_count: u32,
}

impl User {
    /// Associated constructor function.
    /// Idiomatically called as `User::new(...)`.
    pub fn new(id: UserId, username: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            id,
            username: username.into(),
            email: email.into(),
            is_active: true,
            sign_in_count: 0,
        }
    }

    /// Method taking `&self` (immutable borrow) to inspect data.
    pub fn is_eligible_for_login(&self) -> bool {
        self.is_active
    }

    /// Method taking `&mut self` (mutable borrow) to modify state in-place.
    pub fn record_sign_in(&mut self) {
        self.sign_in_count += 1;
    }

    /// Method taking `self` (ownership transfer) to consume/transform the instance.
    pub fn deactivate(mut self) -> Self {
        self.is_active = false;
        self
    }
}

// --- 2. Real-World Backend Pattern: The Builder Pattern ---

/// Database configuration struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub database_name: String,
    pub max_connections: u32,
    pub timeout_seconds: u32,
}

/// Builder for `DatabaseConfig` allowing flexible and safe initialization.
#[derive(Debug, Default)]
pub struct DatabaseConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
    database_name: Option<String>,
    max_connections: Option<u32>,
    timeout_seconds: Option<u32>,
}

impl DatabaseConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn database(mut self, db_name: impl Into<String>) -> Self {
        self.database_name = Some(db_name.into());
        self
    }

    pub fn max_connections(mut self, max: u32) -> Self {
        self.max_connections = Some(max);
        self
    }

    pub fn timeout_seconds(mut self, timeout: u32) -> Self {
        self.timeout_seconds = Some(timeout);
        self
    }

    /// Builds the configuration, applying production defaults and validation.
    pub fn build(self) -> Result<DatabaseConfig, &'static str> {
        let host = self.host.unwrap_or_else(|| "127.0.0.1".to_string());
        let port = self.port.unwrap_or(5432);
        let database_name = self.database_name.ok_or("Database name is required")?;
        let max_connections = self.max_connections.unwrap_or(10);
        let timeout_seconds = self.timeout_seconds.unwrap_or(30);

        if max_connections == 0 {
            return Err("max_connections must be greater than 0");
        }

        Ok(DatabaseConfig {
            host,
            port,
            database_name,
            max_connections,
            timeout_seconds,
        })
    }
}

fn main() {
    println!("=== Chapter 5: Structs, Methods & Builders ===");

    // Creating and mutating a domain entity
    let mut alice = User::new(UserId(101), "alice", "alice@example.com");
    println!("Created user: {alice:?}");

    alice.record_sign_in();
    println!("After sign-in: sign_in_count = {}", alice.sign_in_count);

    let deactivated_alice = alice.deactivate();
    println!(
        "Deactivated user eligible? {}",
        deactivated_alice.is_eligible_for_login()
    );

    // Using the Builder Pattern for Database Configuration
    let db_config = DatabaseConfigBuilder::new()
        .host("postgres.internal.net")
        .database("production_db")
        .max_connections(25)
        .build()
        .expect("Valid database configuration");

    println!("\nConstructed Database Config via Builder:\n{db_config:#?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_lifecycle() {
        let mut user = User::new(UserId(1), "bob", "bob@example.com");
        assert!(user.is_eligible_for_login());
        assert_eq!(user.sign_in_count, 0);

        user.record_sign_in();
        assert_eq!(user.sign_in_count, 1);

        let deactivated = user.deactivate();
        assert!(!deactivated.is_eligible_for_login());
    }

    #[test]
    fn test_database_config_builder_success() {
        let config = DatabaseConfigBuilder::new()
            .database("app_db")
            .port(5433)
            .build()
            .expect("Builder should succeed");

        assert_eq!(config.host, "127.0.0.1"); // Default
        assert_eq!(config.port, 5433);
        assert_eq!(config.database_name, "app_db");
        assert_eq!(config.max_connections, 10); // Default
    }

    #[test]
    fn test_database_config_builder_missing_db() {
        let result = DatabaseConfigBuilder::new().build();
        assert_eq!(result, Err("Database name is required"));
    }

    #[test]
    fn test_database_config_builder_invalid_max_connections() {
        let result = DatabaseConfigBuilder::new()
            .database("app_db")
            .max_connections(0)
            .build();
        assert_eq!(result, Err("max_connections must be greater than 0"));
    }
}
