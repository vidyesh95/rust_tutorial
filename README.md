# Rust tutorial <img src="https://github.com/vidyesh95/rust_tutorial/assets/13806493/f3140e94-997a-4df2-af6c-b0f4507975cb" alt="Rust mascot" width="40"/>

Tutorial repository following [The Rust Programming Language Book](https://doc.rust-lang.org/book/),
3rd edition (2026), which assumes **Rust 1.98+** and **Edition 2024**.

Current toolchain: **rustc / cargo 1.100.0**, `edition = "2024"`, organized as a Cargo workspace.

> **Status: Phase 1 & 2 Complete.** Chapters 1–11, 13, 15, 16 are fully covered with working examples, 30 automated tests, and backend design patterns.
> See [ROADMAP.md](ROADMAP.md) for the master curriculum roadmap from language fundamentals to modern async backend services (`Tokio`, `Axum`, `SQLx`).

---

## Contents

- [What's in the repo](#whats-in-the-repo)
- [Book coverage map](#book-coverage-map)
- [Missing concepts, grouped](#missing-concepts-grouped)
- [Why these matter for backend](#why-these-matter-for-backend)
- [Learning path](#learning-path)
- [Beyond the book: the backend stack](#beyond-the-book-the-backend-stack)
- [Capstone project](#capstone-project)
- [How to study this book](#how-to-study-this-book)
- [Workspace commands](#workspace-commands)
- [Known follow-ups](#known-follow-ups)

---

## What's in the repo

| Chapter / Topic | Package | Status | Command |
|---|---|---|---|
| Getting Started | `rust_tutorial` | Done | `cargo run -p rust_tutorial` |
| Hello Cargo | `hello_cargo` | Done | `cargo run -p hello_cargo` |
| Variables & Mutability | `variables` | Done | `cargo run -p variables` |
| Data Types | `datatypes` | Done | `cargo run -p datatypes` |
| Functions | `functions` | Done | `cargo run -p functions` |
| Control Flow (Conditions) | `conditions` | Done | `cargo run -p conditions` |
| Loops | `loops` | Done | `cargo run -p loops` |
| Operators (Appendix B) | `operators` | Done | `cargo run -p operators` |
| Ownership (Basics) | `ownership` | Partial (4.1 only) | `cargo run -p ownership` |
| Guessing Game | `guessing_game` | Done | `cargo run -p guessing_game` |
| **Borrowing & Slices** | `ownership_borrowing` | **Done (Phase 1)** | `cargo run -p ownership_borrowing` |
| **Structs, Methods & Builders** | `structs_methods` | **Done (Phase 1)** | `cargo run -p structs_methods` |
| **Enums & Pattern Matching** | `enums_matching` | **Done (Phase 1)** | `cargo run -p enums_matching` |
| **Common Collections** | `collections` | **Done (Phase 1)** | `cargo run -p collections` |
| **Error Handling & HTTP Status** | `error_handling` | **Done (Phase 1)** | `cargo run -p error_handling` |
| **Traits, Generics & Repositories** | `traits_generics` | **Done (Phase 2)** | `cargo run -p traits_generics` |
| **Iterators, Closures & Streams** | `iterators_closures` | **Done (Phase 2)** | `cargo run -p iterators_closures` |
| **Smart Pointers, RAII & Deref** | `smart_pointers` | **Done (Phase 2)** | `cargo run -p smart_pointers` |
| **Concurrency, Threads & Mutexes** | `concurrency_threads` | **Done (Phase 2)** | `cargo run -p concurrency_threads` |

Everything else in the book has no crate yet. New chapters should follow the same pattern: one
package per chapter, added to `members` in the root `Cargo.toml`.

---

## Book coverage map

Legend: **Done** = working code plus notes · **Partial** = some of the chapter exists · **Missing** =
not covered anywhere in this repo.

### Part I — Getting started

| Book chapter | Status | Notes |
|---|---|---|
| [1. Getting Started](https://doc.rust-lang.org/book/ch01-00-getting-started.html) | Done | `hello_cargo`, root binary |
| [2. Programming a Guessing Game](https://doc.rust-lang.org/book/ch02-00-guessing-game-tutorial.html) | Done | Uses `rand` 0.10 (`rand::random_range`) |

### Part II — Core language

| Book chapter | Status | Notes |
|---|---|---|
| [3. Common Programming Concepts](https://doc.rust-lang.org/book/ch03-00-common-programming-concepts.html) | Done | Split across `variables`, `datatypes`, `functions`, `conditions`, `loops` |
| [3.4 Comments](https://doc.rust-lang.org/book/ch03-04-comments.html) | Done | Standard `//` and `///` doc comments used across crates |
| [4. Understanding Ownership](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html) | Done | Rules, stack vs heap, move vs copy semantics |
| [4.2 References and Borrowing](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html) | **Done** | Implemented in `ownership_borrowing` (`&T`, `&mut T`, borrow rules) |
| [4.3 The Slice Type](https://doc.rust-lang.org/book/ch04-03-slices.html) | **Done** | Implemented in `ownership_borrowing` (zero-copy HTTP parser) |
| [5. Using Structs](https://doc.rust-lang.org/book/ch05-00-structs.html) | **Done** | Implemented in `structs_methods` (entities, newtype, Builder pattern) |
| [6. Enums and Pattern Matching](https://doc.rust-lang.org/book/ch06-00-enums.html) | **Done** | Implemented in `enums_matching` (state machines, `Option`, `let-else`) |
| [7. Packages, Crates, and Modules](https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html) | **Done** | Multi-crate workspace architecture |
| [8. Common Collections](https://doc.rust-lang.org/book/ch08-00-common-collections.html) | **Done** | Implemented in `collections` (`Vec`, UTF-8 `String`, `HashMap` metrics) |
| [9. Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html) | **Done** | Implemented in `error_handling` (`Result`, `?`, `ApiError`, HTTP codes) |

### Part III — Abstraction and projects

| Book chapter | Status | Notes |
|---|---|---|
| [10. Generics, Traits, Lifetimes](https://doc.rust-lang.org/book/ch10-00-generics.html) | **Done** | Implemented in `traits_generics` (Repository Pattern, DI) |
| [11. Writing Automated Tests](https://doc.rust-lang.org/book/ch11-00-testing.html) | **Done** | 30 unit tests across all workspace packages |
| [12. An I/O Project (minigrep)](https://doc.rust-lang.org/book/ch12-00-an-io-project.html) | **Missing** | First real program: args, files, env vars, stderr |
| [13. Iterators and Closures](https://doc.rust-lang.org/book/ch13-00-functional-features.html) | **Done** | Implemented in `iterators_closures` (pipelines, pagination) |
| [14. More about Cargo](https://doc.rust-lang.org/book/ch14-00-more-about-cargo.html) | Partial | Workspace is *used*, but profiles, features, publishing are not studied |

### Part IV — Advanced topics and the final project

| Book chapter | Status | Notes |
|---|---|---|
| [15. Smart Pointers](https://doc.rust-lang.org/book/ch15-00-smart-pointers.html) | **Done** | Implemented in `smart_pointers` (Box, Drop, Deref, Rc, Arc, RefCell) |
| [16. Fearless Concurrency](https://doc.rust-lang.org/book/ch16-00-concurrency.html) | **Done** | Implemented in `concurrency_threads` (threads, mpsc, Arc<Mutex<T>>) |
| [17. Async, Await, Futures, Streams](https://doc.rust-lang.org/book/ch17-00-async-await.html) | **Phase 4** | Async runtime & Tokio scheduled in Phase 4 |
| [18. OOP Features](https://doc.rust-lang.org/book/ch18-00-oop.html) | **Done** | Implemented trait objects (`dyn Trait`) in `traits_generics` |
| [19. Patterns and Matching](https://doc.rust-lang.org/book/ch19-00-patterns.html) | Partial | Covered extensively in `enums_matching` |
| [20. Advanced Features](https://doc.rust-lang.org/book/ch20-00-advanced-features.html) | **Deferred** | `unsafe`, macros, advanced traits/types |
| [21. Final Project: Web Server](https://doc.rust-lang.org/book/ch21-00-final-project-a-web-server.html) | **Phase 3** | Scheduled for Phase 3 (raw multithreaded TCP server) |

### Appendices

| Appendix | Status | Notes |
|---|---|---|
| [A. Keywords](https://doc.rust-lang.org/book/appendix-01-keywords.html) | Missing | Reference, read as needed |
| [B. Operators and Symbols](https://doc.rust-lang.org/book/appendix-02-operators.html) | Partial | `operators` covers arithmetic/comparison/logical/bitwise only |
| [C. Derivable Traits](https://doc.rust-lang.org/book/appendix-03-derivable-traits.html) | **Done** | `Debug`, `Clone`, `PartialEq`, `Eq`, `Default` used across all crates |
| [D. Useful Development Tools](https://doc.rust-lang.org/book/appendix-04-useful-development-tools.html) | **Done** | `fmt`, `clippy`, and `cargo test` wired up and strictly enforced |
| [E. Editions](https://doc.rust-lang.org/book/appendix-05-editions.html) | **Done** | Rust Edition 2024 active with `let-else` guards |

**Coverage: 15 of 21 chapters complete with 30 comprehensive unit tests and backend design patterns.**

---

## Missing concepts, grouped

These are the ideas the book teaches that this repo does not demonstrate anywhere. The list is
ordered by how much it will hurt to skip.

**Ownership and memory**
- Borrowing: `&T` and `&mut T`, and the rule "one mutable XOR many immutable references"
- Move vs `Copy` vs `Clone`; why `String` moves and `i32` copies
- Slices: `&str` vs `String`, `&[T]` vs `Vec<T>`
- Lifetimes: elision, `'a`, structs that hold references
- `Deref` and `Drop`

**Modelling data**
- Structs, tuple structs, `impl` blocks, methods, associated functions
- Enums, `Option<T>`, `Result<T, E>`, `match`, `if let`, `let ... else`
- Pattern destructuring, guards, `@` bindings
- Traits: definitions, bounds, default methods, `impl Trait`, `dyn Trait`
- Generics and monomorphization
- `derive`: `Debug`, `Clone`, `PartialEq`, `Eq`, `Hash`

**Code organization**
- Package vs crate vs module; `pub`, `use`, `super`, `crate`, `self`
- `lib.rs` + `main.rs` split, integration tests in `tests/`
- Workspaces, features, release profiles, publishing

**Std library and data handling**
- `Vec<T>`, `String`/UTF-8, `HashMap`, `BTreeMap`, `HashSet`
- Iterators: `map`, `filter`, `fold`, `collect`, laziness, `impl Iterator`
- Closures: captures, `move`, `Fn`/`FnMut`/`FnOnce`

**Errors**
- `panic!` vs recoverable `Result`
- The `?` operator and `From` conversions between error types
- Custom error enums; `Box<dyn Error>`; `thiserror` for libraries, `anyhow` for binaries
- When panicking is correct (tests, invariants) and when it is not (request handling)

**Concurrency and async**
- `thread::spawn`, `move` closures, `mpsc` channels
- `Arc<Mutex<T>>` and shared state
- `Send`/`Sync`, and why the compiler rejects some sharing
- Futures, `async`/`await`, executors, `tokio::spawn`, `JoinHandle`
- Streams, `select!`, cancellation, and never blocking inside async

**Engineering practice**
- Unit tests (`#[cfg(test)]`), integration tests, doc tests
- `cargo test`, `cargo bench`, `cargo doc`
- Clippy lints as a teaching tool, `rustfmt`, `rust-analyzer`
- `unsafe`, macros, advanced traits — deliberately last: you need them for FFI, performance,
  and DSLs, not for a first service

---

## Why these matter for backend

| Concept | Book | What it buys you in a service |
|---|---|---|
| Ownership and borrowing | 4 | Deciding when to borrow vs clone in handlers; no GC pauses; safe shared state |
| `Result` and `?` | 6, 9 | Every DB call, parse, and network call can fail; mapping errors to HTTP status codes |
| Structs and enums | 5, 6 | Domain models, request/response types, state machines (`Pending`/`Paid`/`Shipped`) |
| Traits | 10 | Repository abstractions, handler extractors, swappable implementations, test doubles |
| Modules and workspaces | 7, 14 | A service is many crates: `api`, `domain`, `db`, `config` |
| Collections | 8 | JSON payloads, in-memory caches, grouping query results |
| Tests | 11 | Integration tests are how you trust a service you cannot step through |
| Iterators and closures | 13 | Transforming rows into responses without index loops |
| Smart pointers | 15 | `Arc` for shared config/state, `Box<dyn Error>` at boundaries |
| Threads and channels | 16 | Background jobs, worker pools, blocking work off the request path |
| Async/await | 17 | Every modern Rust web framework is async; concurrency without a thread per request |
| Trait objects | 18 | Dynamic dispatch for middleware, storage backends, error types |
| Web server project | 21 | HTTP, routing, thread pools, graceful shutdown — before reaching for a framework |

The book gets you to "I can write Rust". Backend engineering adds the other half: HTTP semantics,
SQL and transactions, connection pooling, auth, observability, deployment. Do not skip the language
half — async Rust on a shaky ownership foundation is miserable.

---

## Learning path

Roughly 12 weeks at 1–2 hours a day. Each phase ends with working code in this repo.

| Phase | Weeks | Book | Build | Outcome |
|---|---|---|---|---|
| 0. Fix the foundation | 1 | 4.2, 4.3 | Rewrite `ownership` with move/clone/borrow/slice demos | You can predict borrow-checker errors |
| 1. Model data | 2–3 | 5, 6, 7 | `structs`, `enums`, `modules` crates; split a crate into `lib.rs` + `main.rs` | You can design types and lay out a crate |
| 2. Collections and errors | 4 | 8, 9 | CLI that reads a file, groups lines into a `HashMap`, returns `Result` with a custom error | No more `.unwrap()` in real code |
| 3. Abstraction and tests | 5–6 | 10, 11, 13 | Generic `Repository` trait + unit and integration tests + iterator refactor | You can write testable, generic code |
| 4. A real program | 7 | 12, 14 | `minigrep` as a workspace member with a `core` library crate | You can structure a multi-crate project |
| 5. Shared state | 8 | 15, 16 | Threaded counter/queue with `Arc<Mutex<T>>` and `mpsc` | You understand `Send`/`Sync` errors |
| 6. Async | 9–10 | 17 | `tokio` exercises: concurrent tasks, `JoinHandle`, streams, timeouts | You can write async code and spot blocking calls |
| 7. HTTP from scratch | 11 | 21 | The book's thread-pool web server, then the same routes in `axum` | You know what a framework hides |
| 8. Capstone | 12+ | 18, 19 as needed | The service below | You can build and ship a small backend |

Chapters 18–20 are reference material: read them when a design question sends you there, not
front-to-back on the first pass.

**Do not** jump to `axum`/`sqlx` before phase 6. Framework tutorials hide ownership behind
extractors and `async` blocks; when the borrow checker complains inside a handler you need the
vocabulary from phases 0–5.

---

## Beyond the book: the backend stack

The book stops at language fundamentals plus a toy HTTP server. It does **not** cover web
frameworks, databases, auth, deployment, or observability. After chapter 21, add:

| Need | Crates / tools | Why |
|---|---|---|
| Async runtime | `tokio` | `#[tokio::main]`, `spawn`, timers, channels |
| HTTP | `axum` (+ `tower`, `tower-http`) | Routing, extractors, middleware, CORS, tracing |
| JSON | `serde`, `serde_json` | Derive `Serialize`/`Deserialize` on your types |
| Database | `sqlx` (Postgres) + `sqlx-cli` | Compile-time checked SQL, pooling, migrations |
| Errors | `thiserror`, `anyhow` | Typed library errors, contextual app errors |
| Config | `dotenvy`, `config`/`figment` | 12-factor config from env |
| Logging | `tracing`, `tracing-subscriber` | Structured logs with request spans |
| Auth | `jsonwebtoken`, `argon2` | JWT sessions, password hashing |
| Validation | `validator`/`garde` | Reject bad input at the boundary |
| HTTP testing | `httpmock`/`wiremock`, `testcontainers` | Test clients and real Postgres in CI |
| API docs | `utoipa` + `swagger-ui` | OpenAPI from the code |
| Dev loop | `cargo-watch`/`bacon`, `cargo-nextest` | Fast feedback |
| Supply chain | `cargo-audit`, `cargo-deny` | Known vulns, license policy |
| Deploy | multi-stage Dockerfile, `docker compose`, health checks | Reproducible images, local parity |

**Backend concepts the book will never teach**, and that you should study alongside the capstone:
HTTP methods/status codes/idempotency, REST resource design, pagination and filtering, SQL schema
design plus indexes and transactions, N+1 queries, connection pooling and pool exhaustion,
migrations, cache invalidation, queue/retry/backoff and idempotency keys, rate limiting,
authentication vs authorization, secret management, structured logs/metrics/traces, health and
readiness probes, graceful shutdown, and CI.

---

## Capstone project

A **task tracker API** is enough surface area to touch every backend concern once:

1. `POST /tasks`, `GET /tasks?status=&limit=&cursor=`, `PATCH /tasks/{id}`, `DELETE /tasks/{id}`
2. Postgres via `sqlx`, schema managed by migrations, one transaction per request where needed
3. `serde` request/response types, validation, and a typed error enum that maps to status codes
4. Bearer-token auth (`jsonwebtoken`) with `argon2` password hashing
5. `tracing` middleware with request IDs; JSON logs in production
6. Graceful shutdown, `/health` and `/ready`, config from environment
7. Integration tests against a real database; `cargo clippy -- -D warnings` in CI
8. `docker compose up` starts Postgres and the API

Do it once with the book's thread-pool server and once with `axum`. The first teaches what HTTP is;
the second is what you would actually ship.

---

## How to study this book

- **Type every example.** Reading Rust creates the illusion of understanding; the borrow checker
  is the exam.
- **Break examples on purpose.** Remove a `&`, clone instead of borrow, move a value twice. Read the
  full compiler error, then `rustc --explain E0382`.
- **One crate per chapter, one commit per chapter.** Keeps this repo a usable reference later.
- **Write the test before the refactor** from chapter 11 onward.
- **Keep `cargo clippy --workspace --all-targets -- -D warnings` green.** The lints teach idiom.
- **Explain each new concept out loud in one sentence.** If you cannot, you have not finished the
  chapter.
- **Expect ownership to take two passes.** Chapter 4 makes sense immediately and then stops making
  sense in chapter 10 (lifetimes) and chapter 17 (async). That is normal; revisit it.

---

## Workspace commands

Run from the repository root:

```shell
# Check all workspace crates
cargo check --workspace

# Run tests across all workspace crates (no tests exist yet; chapter 11 adds them)
cargo test --workspace

# Lint all workspace crates with Clippy
cargo clippy --workspace --all-targets -- -D warnings

# Format all workspace code
cargo fmt --all

# Build docs for the workspace
cargo doc --workspace --no-deps --open
```

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

---

## Known follow-ups

Small inconsistencies to fix as the repo grows:

1. `ownership` explains the rules but never demonstrates move, clone, or borrowing — phase 0 of the
   plan above.
2. `cargo test --workspace` is advertised but no crate contains a `#[test]` function yet.
3. The workspace sets `resolver = "2"`; Edition 2024 implies resolver 3, so `resolver = "3"` is the
   consistent choice.
4. `rust-version = "1.100"` is configured in `Cargo.toml` as the minimum supported Rust version (MSRV).
5. The root package `rust_tutorial` duplicates `hello_cargo`; consider making the root a library
   crate that the per-chapter binaries depend on, or drop it.
