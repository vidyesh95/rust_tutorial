//! Integration test (chapter 11): a separate crate that can only use `modules`' public API.
//!
//! Unlike `#[cfg(test)]` unit tests, this file cannot see private modules, `pub(crate)` items, or
//! private fields. If it compiles, the public surface is genuinely usable.

use modules::User;
use modules::api;

#[test]
fn creates_a_user_through_the_public_api() {
    let user = api::handlers::create_user(7, "  Grace Hopper ").expect("valid user");

    assert_eq!(user.id(), 7);
    assert_eq!(user.name(), "grace hopper");
}

#[test]
fn invalid_input_becomes_a_client_error() {
    let response = api::handlers::create_user(8, "   ").expect_err("blank name is rejected");
    assert!(response.starts_with("400"), "got: {response}");
}

#[test]
fn the_re_export_is_reachable_from_the_crate_root() {
    let user = User::new(9, "Ada").expect("valid user");
    assert_eq!(user.name(), "ada");
}

#[test]
fn routes_are_public() {
    assert_eq!(api::handlers::route("/health"), "ok");
}
