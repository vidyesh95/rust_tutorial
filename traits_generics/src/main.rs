//! # Chapter 10: Generics, Traits, Lifetimes & The Repository Pattern
//!
//! ## Key Backend Insights:
//! - **Traits as Interfaces**: In Rust, traits define shared contracts. Unlike OOP interfaces,
//!   traits can be implemented for external types, contain default methods, and support zero-cost static dispatch.
//! - **Dependency Injection**: We can decouple business logic from the database layer by writing services
//!   that accept generic types bounded by a trait (e.g., `UserService<R: UserRepository>`).
//! - **Static vs Dynamic Dispatch**:
//!   - `impl Trait` / `<T: Trait>` uses **monomorphization** (zero runtime cost, inlined by compiler).
//!   - `dyn Trait` (Trait Objects) uses virtual method tables (`vtable`) for runtime polymorphism.

use std::collections::HashMap;

// --- 1. Basic Generics & Trait Bounds ---

/// Generic function that works on any slice where elements implement `PartialEq` and `Clone`.
pub fn find_first_index<T: PartialEq>(items: &[T], target: &T) -> Option<usize> {
    for (idx, item) in items.iter().enumerate() {
        if item == target {
            return Some(idx);
        }
    }
    None
}

// --- 2. Real-World Backend Pattern: The Repository Pattern ---

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub email: String,
}

/// Abstract contract for user persistence.
/// Real backends implement this with Postgres, MySQL, or Mongo;
/// Unit tests implement this with an in-memory HashMap.
pub trait UserRepository {
    fn create(&mut self, user: User) -> Result<(), &'static str>;
    fn find_by_id(&self, id: u64) -> Option<&User>;
    fn find_by_username(&self, username: &str) -> Option<&User>;
    fn count(&self) -> usize;

    /// Default method implementation provided by the trait.
    fn is_empty(&self) -> bool {
        self.count() == 0
    }
}

/// In-Memory implementation of `UserRepository` for fast local testing.
#[derive(Debug, Default)]
pub struct InMemoryUserRepository {
    users: HashMap<u64, User>,
}

impl InMemoryUserRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl UserRepository for InMemoryUserRepository {
    fn create(&mut self, user: User) -> Result<(), &'static str> {
        if self.users.contains_key(&user.id) {
            return Err("User with this ID already exists");
        }
        self.users.insert(user.id, user);
        Ok(())
    }

    fn find_by_id(&self, id: u64) -> Option<&User> {
        self.users.get(&id)
    }

    fn find_by_username(&self, username: &str) -> Option<&User> {
        self.users.values().find(|u| u.username == username)
    }

    fn count(&self) -> usize {
        self.users.len()
    }
}

// --- 3. Dependency Injection in Services ---

/// Business logic service decoupled from the database via static dispatch (`R: UserRepository`).
pub struct UserService<R: UserRepository> {
    repository: R,
}

impl<R: UserRepository> UserService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub fn register_user(
        &mut self,
        id: u64,
        username: &str,
        email: &str,
    ) -> Result<User, &'static str> {
        if username.trim().is_empty() {
            return Err("Username cannot be empty");
        }
        if self.repository.find_by_username(username).is_some() {
            return Err("Username is already taken");
        }

        let user = User {
            id,
            username: username.to_string(),
            email: email.to_string(),
        };

        self.repository.create(user.clone())?;
        Ok(user)
    }

    pub fn get_user(&self, id: u64) -> Option<&User> {
        self.repository.find_by_id(id)
    }
}

// --- 4. Dynamic Dispatch (Trait Objects: `Box<dyn Trait>`) ---

/// Demonstrates runtime polymorphism.
/// When you need a heterogeneous collection of services or to hide concrete types behind a pointer.
pub fn print_repository_summary(repo: &dyn UserRepository) {
    println!(
        "Repository has {} registered users (is_empty: {})",
        repo.count(),
        repo.is_empty()
    );
}

fn main() {
    println!("=== Chapter 10: Generics, Traits & The Repository Pattern ===");

    // 1. Generic function demo
    let roles = vec!["reader", "editor", "admin"];
    println!("Index of 'admin': {:?}", find_first_index(&roles, &"admin"));
    println!("Index of 'guest': {:?}", find_first_index(&roles, &"guest"));

    // 2. Repository Pattern & Dependency Injection demo
    let repo = InMemoryUserRepository::new();
    let mut service = UserService::new(repo);

    let u1 = service
        .register_user(1, "alice", "alice@example.com")
        .expect("Alice registration should succeed");
    println!("Successfully registered: {u1:?}");

    let u2 = service
        .register_user(2, "bob", "bob@example.com")
        .expect("Bob registration should succeed");
    println!("Successfully registered: {u2:?}");

    // Test duplicate username validation
    let duplicate = service.register_user(3, "alice", "alice_alt@example.com");
    println!("Duplicate registration result: {duplicate:?}");

    // 3. Trait Object dynamic dispatch demo
    print_repository_summary(&service.repository);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_first_index() {
        let numbers = [10, 20, 30, 40];
        assert_eq!(find_first_index(&numbers, &30), Some(2));
        assert_eq!(find_first_index(&numbers, &99), None);
    }

    #[test]
    fn test_user_service_registration() {
        let repo = InMemoryUserRepository::new();
        let mut service = UserService::new(repo);

        let res = service.register_user(1, "john", "john@example.com");
        assert!(res.is_ok());
        assert_eq!(service.get_user(1).unwrap().username, "john");

        // Duplicate test
        let dup = service.register_user(2, "john", "another@example.com");
        assert_eq!(dup, Err("Username is already taken"));
    }

    #[test]
    fn test_empty_username_validation() {
        let repo = InMemoryUserRepository::new();
        let mut service = UserService::new(repo);

        let res = service.register_user(1, "   ", "blank@example.com");
        assert_eq!(res, Err("Username cannot be empty"));
    }
}
