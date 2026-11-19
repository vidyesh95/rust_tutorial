# Rust tutorial <img src="https://github.com/vidyesh95/rust_tutorial/assets/13806493/f3140e94-997a-4df2-af6c-b0f4507975cb" alt="Rust mascot" width="40"/>

Tutorial repository following [The Rust Programming Language Book](https://doc.rust-lang.org/book/),
3rd edition (2026), which assumes **Rust 1.98+** and **Edition 2024**.

Current toolchain: **rustc / cargo 1.100.0** (pinned in `rust-toolchain.toml`), `edition = "2024"`,
organized as a Cargo workspace.

---

## Contents

- [Interview & Backend Masterclass (INTERVIEW_PREP.md)](INTERVIEW_PREP.md)
- [Relational Database & SQL Handbook (SQL_RELATIONAL_DATABASE_GUIDE.md)](SQL_RELATIONAL_DATABASE_GUIDE.md)
- [What's in the repo](#whats-in-the-repo)
- [Book coverage map](#book-coverage-map)
- [What's still missing](#whats-still-missing)
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
| Guessing Game | `guessing_game` | Done | `cargo run -p guessing_game` |
| Ownership, Borrowing & Slices (4.1–4.3) | `ownership` | Done, 10 tests | `cargo run -p ownership` |
| Applied Borrowing (zero-copy parsing) | `ownership_borrowing` | Done, 4 tests | `cargo run -p ownership_borrowing` |
| Structs, Methods & Builders | `structs_methods` | Done, 4 tests | `cargo run -p structs_methods` |
| Enums & Pattern Matching | `enums_matching` | Done, 4 tests | `cargo run -p enums_matching` |
| Modules, Crates & Visibility (7) | `modules` | Done, 10 tests (unit + integration + doc) | `cargo run -p modules` |
| Common Collections | `collections` | Done, 2 tests | `cargo run -p collections` |
| Error Handling & HTTP Status | `error_handling` | Done, 4 tests | `cargo run -p error_handling` |
| Traits, Generics & Repositories | `traits_generics` | Done, 3 tests | `cargo run -p traits_generics` |
| Lifetimes (10.3) | `lifetimes` | Done, 5 tests | `cargo run -p lifetimes` |
| Iterators, Closures & Pipelines | `iterators_closures` | Done, 3 tests | `cargo run -p iterators_closures` |
| Smart Pointers, RAII & Deref | `smart_pointers` | Done, 3 tests | `cargo run -p smart_pointers` |
| Concurrency, Threads & Mutexes | `concurrency_threads` | Done, 2 tests | `cargo run -p concurrency_threads` |
| Multithreaded Web Server (21) | `multithreaded_webserver` | Done, 9 tests | `cargo run -p multithreaded_webserver` |
| Async Programming & Tokio (17) | `async_tokio` | Done, 5 tests | `cargo run -p async_tokio` |
| Production REST API & Swagger UI (Axum + SQLite + Auth + Tx) | `backend_api_service` | Done, 7 tests | `cargo run -p backend_api_service` |

`modules` is the chapter 7 reference: `lib.rs` + `main.rs`, `mod`/`pub`/`use`, a private module, a
re-export, and a real `tests/` directory. `multithreaded_webserver` is the chapter 21 reference: TCP
listener, HTTP/1.1 request line, thread pool, `Drop`-based cleanup.

`ownership` teaches the chapter 4 mechanics (move/copy/clone, `&T`/`&mut T`, slices);
`ownership_borrowing` applies them to parsing an HTTP request line with zero heap allocations.

Everything else in the book has no crate yet: chapters 12, 17, and 20, plus the deeper Cargo and
pattern chapters.

---

## Book coverage map

Legend: **Done** = working code plus notes · **Partial** = the concept appears but the chapter is not
fully covered · **Missing** = not covered anywhere in this repo.

### Part I — Getting started

| Book chapter | Status | Notes |
|---|---|---|
| [1. Getting Started](https://doc.rust-lang.org/book/ch01-00-getting-started.html) | Done | `hello_cargo`, root binary |
| [2. Programming a Guessing Game](https://doc.rust-lang.org/book/ch02-00-guessing-game-tutorial.html) | Done | Uses `rand` 0.10 (`rand::random_range`) |

### Part II — Core language

| Book chapter | Status | Notes |
|---|---|---|
| [3. Common Programming Concepts](https://doc.rust-lang.org/book/ch03-00-common-programming-concepts.html) | Done | Split across `variables`, `datatypes`, `functions`, `conditions`, `loops` |
| [3.4 Comments](https://doc.rust-lang.org/book/ch03-04-comments.html) | Done | `//` and `///` doc comments used across crates |
| [4. Understanding Ownership](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html) | Done | `ownership`: rules, stack/heap, move, `Copy`, `Clone`, borrowing, slices |
| [4.2 References and Borrowing](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html) | Done | `ownership` (rules, NLL) and `ownership_borrowing` (`&str` handlers) |
| [4.3 The Slice Type](https://doc.rust-lang.org/book/ch04-03-slices.html) | Done | `first_word`, array slices, zero-copy HTTP parser |
| [5. Using Structs](https://doc.rust-lang.org/book/ch05-00-structs.html) | Done | `structs_methods`: entities, newtype, Builder pattern |
| [6. Enums and Pattern Matching](https://doc.rust-lang.org/book/ch06-00-enums.html) | Done | `enums_matching`: state machines, `Option`, `let ... else` |
| [7. Packages, Crates, and Modules](https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html) | Done | `modules`: `lib.rs` + `main.rs`, `mod`/`pub`/`use`, private module, re-export, `tests/` |
| [8. Common Collections](https://doc.rust-lang.org/book/ch08-00-common-collections.html) | Done | `collections`: `Vec`, UTF-8 `String`, `HashMap` metrics |
| [9. Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html) | Done | `error_handling`: `Result`, `?`, `ApiError`, HTTP status mapping |

### Part III — Abstraction and projects

| Book chapter | Status | Notes |
|---|---|---|
| [10. Generics, Traits, Lifetimes](https://doc.rust-lang.org/book/ch10-00-generics.html) | Done | `traits_generics` (generics, traits, `dyn Trait`, DI) and `lifetimes` (`'a`, elision, borrowing structs) |
| [11. Writing Automated Tests](https://doc.rust-lang.org/book/ch11-00-testing.html) | Done | 70 tests: `#[cfg(test)]` units, `tests/` integration suites, and doc tests |
| [12. An I/O Project (minigrep)](https://doc.rust-lang.org/book/ch12-00-an-io-project.html) | Missing | CLI project: args, files, env vars, stderr |
| [13. Iterators and Closures](https://doc.rust-lang.org/book/ch13-00-functional-features.html) | Done | `iterators_closures`: pipelines, `move` closures, pagination |
| [14. More about Cargo](https://doc.rust-lang.org/book/ch14-00-more-about-cargo.html) | Partial | Workspace is used; profiles, features, publishing not studied |

### Part IV — Advanced topics and the final project

| Book chapter | Status | Notes |
|---|---|---|
| [15. Smart Pointers](https://doc.rust-lang.org/book/ch15-00-smart-pointers.html) | Done | `smart_pointers`: `Box`, `Deref`, `Drop`, `Rc`, `Arc`, `RefCell` |
| [16. Fearless Concurrency](https://doc.rust-lang.org/book/ch16-00-concurrency.html) | Done | `concurrency_threads`: `thread::spawn`, `mpsc`, `Arc<Mutex<T>>` |
| [17. Async, Await, Futures, Streams](https://doc.rust-lang.org/book/ch17-00-async-await.html) | **Done** | `async_tokio`: Tokio runtime, `tokio::spawn`, `join!`, timeouts, async mpsc |
| [18. OOP Features](https://doc.rust-lang.org/book/ch18-00-oop.html) | Partial | `dyn Trait` in `traits_generics`; the OOP state pattern is not implemented |
| [19. Patterns and Matching](https://doc.rust-lang.org/book/ch19-00-patterns.html) | Partial | Patterns used throughout `enums_matching`; the chapter's syntax survey is not done |
| [20. Advanced Features](https://doc.rust-lang.org/book/ch20-00-advanced-features.html) | Missing | Deferred: `unsafe`, macros, advanced traits/types |
| [21. Final Project: Web Server](https://doc.rust-lang.org/book/ch21-00-final-project-a-web-server.html) | **Done** | `multithreaded_webserver` (Ch21 thread pool) & `backend_api_service` (Axum + SQLx SQLite) |

### Appendices

| Appendix | Status | Notes |
|---|---|---|
| [A. Keywords](https://doc.rust-lang.org/book/appendix-01-keywords.html) | Missing | Reference, read as needed |
| [B. Operators and Symbols](https://doc.rust-lang.org/book/appendix-02-operators.html) | Partial | `operators` covers arithmetic/comparison/logical/bitwise only |
| [C. Derivable Traits](https://doc.rust-lang.org/book/appendix-03-derivable-traits.html) | Done | `Debug`, `Clone`, `PartialEq`, `Eq`, `Default`, `Serialize`, `Deserialize` |
| [D. Useful Development Tools](https://doc.rust-lang.org/book/appendix-04-useful-development-tools.html) | Done | `fmt`, `clippy`, `cargo test` wired up and enforced |
| [E. Editions](https://doc.rust-lang.org/book/appendix-05-editions.html) | Partial | Edition 2024 is configured; the edition system itself is not explained |

**Coverage: 16 of 21 chapters complete. 70 automated tests pass.**

---

## What's still missing

### Book material not built yet

| Gap | Why it matters |
|---|---|
| Chapter 12 — minigrep | First real program: `env::args`, file I/O, env vars, stderr, TDD refactor |
| Chapter 17 — async/await, futures, streams | Every production Rust web framework is async; this is the biggest remaining language gap |
| Chapter 21 — multithreaded web server | TCP, HTTP/1.1 parsing, thread pool, graceful shutdown — do it before Axum |
| Chapter 20 — `unsafe`, macros, advanced traits | Deferred on purpose: needed for FFI and libraries, not a first service |

### Concepts inside partial chapters

- **Modules (ch7):** no `mod`, `pub`, `use`, `super`, or `lib.rs` + `main.rs` split yet. The workspace
  is real, but a service crate needs a module tree.
- **Lifetimes (ch10.3):** no `'a` annotations on functions or structs, and no lifetime elision rules.
  You need this before structs that borrow data and before async.
- **Integration tests (ch11):** tests exist only as `#[cfg(test)]` modules; no `tests/` directory and
  no doc tests. A backend needs integration tests against a running service.
- **Cargo (ch14):** release profiles, features, and publishing are not studied.
- **OOP state pattern (ch18)** and the full pattern syntax survey (ch19).

### The backend layer the book never covers

JSON (`serde`), SQL and pooling (`sqlx`), migrations, auth, config, structured logging, graceful
shutdown, Docker, CI, and load/soak testing. Those are roadmap phase 4 and the capstone below.

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

Aligned with the four phases in [ROADMAP.md](ROADMAP.md). Roughly 14 weeks at 1–2 hours a day.

| Roadmap phase | Weeks | Covers | Status |
|---|---|---|---|
| **1. Language core** | 1–6 | Ch 1–9: syntax, ownership, structs, enums, collections, errors | Complete |
| **2. Abstractions and concurrency** | 5–8 | Ch 10, 11, 13, 15, 16: traits, tests, iterators, smart pointers, threads | Complete (lifetimes, integration tests open) |
| **3. Network fundamentals** | 9–10 | Ch 21: TCP listener, HTTP/1.1, thread pool, graceful shutdown | **Next** |
| **4. Production backend** | 11–14 | Ch 17 + Tokio, Axum, Serde, SQLx; capstone | Planned |

**Phase 1 — language core.** Build the guessing game, then the ownership/borrowing crates, then model
users and orders with structs and enums. Outcome: you can predict borrow-checker errors and design
domain types.

**Phase 2 — abstractions and concurrency.** Write a `Repository` trait, test it with a fake, refactor
loops into iterator chains, share state with `Arc<Mutex<T>>`, and pass work between threads with
`mpsc`. Outcome: you can write testable, generic, thread-safe code.

**Phase 3 — network fundamentals.** Build `multithreaded_webserver`: `TcpListener`, parse the request
line and headers, return responses, add a fixed-size thread pool, then graceful shutdown and `Drop`.
Outcome: you know what Axum hides. Do this before touching a framework.

**Phase 4 — production backend.** Learn futures and `async`/`await` with Tokio, then Axum routing and
extractors, Serde JSON, and SQLx against Postgres. Finish with the capstone below.

Order matters: async on top of unfinished ownership is the usual failure mode. Chapter 21 before
Axum, chapter 17 before SQLx, and no skipping chapter 10.

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

# Run all 39 unit tests
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

1. **Lifetimes (10.3):** add `'a` annotations, elision, and a struct that borrows its data. This is
   the last piece of chapter 10 and a prerequisite for async.
2. **Module layout (7):** split one crate into `lib.rs` + `mod` files to demonstrate `pub`, `use`, and
   privacy. The workspace exists, but no module tree does.
3. **Integration tests (11):** add a `tests/` directory with a test that exercises a public API.
4. **Chapter 12, 17, 21:** minigrep, async/Tokio, and the TCP server, per the roadmap.
5. **`resolver = "2"`:** Edition 2024 implies resolver 3; `resolver = "3"` is the consistent choice.
6. **Root package:** `rust_tutorial` duplicates `hello_cargo`; make the root a library crate or drop
   it.
