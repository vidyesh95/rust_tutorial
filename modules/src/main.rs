//! Binary entry point.
//!
//! This file is not the crate root: it is a separate binary target that links against the library
//! defined in `lib.rs`. That is why it imports `modules::...` instead of using `crate::...`.

use modules::User;
use modules::api::handlers::{create_user, route};

fn main() {
    println!("modules crate demo");

    match create_user(1, "  Ada Lovelace  ") {
        Ok(user) => println!("created user: id = {}, name = {:?}", user.id(), user.name()),
        Err(response) => println!("{response}"),
    }

    match create_user(2, "   ") {
        Ok(user) => println!("created user: {user:?}"),
        Err(response) => println!("{response}"),
    }

    println!("GET /health -> {}", route("/health"));
    println!("GET /missing -> {}", route("/missing"));

    // `User` is re-exported from the crate root by `pub use domain::User;`.
    let user = User::new(3, "Grace Hopper").expect("valid name");
    println!("re-exported User: {user:?}");
}
