# Rust tutorial <img src="https://github.com/vidyesh95/rust_tutorial/assets/13806493/f3140e94-997a-4df2-af6c-b0f4507975cb" alt="Rust mascot" width="40"/>

Tutorial repository following [The Rust Programming Language Book](https://doc.rust-lang.org/book/).

Updated for **Rust / Cargo 1.100.0** and **Rust Edition 2024** configured as a Cargo Workspace.

---

## Workspace Structure

The project is organized as a Cargo Workspace with package inheritance:

| Chapter / Topic | Package Name | Command |
|---|---|---|
| Getting Started | `rust_tutorial` | `cargo run -p rust_tutorial` |
| Hello Cargo | `hello_cargo` | `cargo run -p hello_cargo` |
| Variables & Mutability | `variables` | `cargo run -p variables` |
| Data Types | `datatypes` | `cargo run -p datatypes` |
| Functions | `functions` | `cargo run -p functions` |
| Control Flow (Conditions) | `conditions` | `cargo run -p conditions` |
| Loops | `loops` | `cargo run -p loops` |
| Operators | `operators` | `cargo run -p operators` |
| Ownership | `ownership` | `cargo run -p ownership` |
| Guessing Game | `guessing_game` | `cargo run -p guessing_game` |

---

## Workspace Commands

Run any of the following commands from the root directory:

```shell
# Check all workspace crates
cargo check --workspace

# Run tests across all workspace crates
cargo test --workspace

# Lint all workspace crates with Clippy
cargo clippy --workspace --all-targets -- -D warnings

# Format all workspace code
cargo fmt --all
```

---

## Manual Execution with `rustc`

If compiling a standalone file directly with the Rust compiler:

```shell
# macOS / Linux
$ rustc src/main.rs
$ ./main

# Windows
> rustc src\main.rs
> .\main.exe
```
