# Rust Backend Engineering Curriculum & Roadmap

Welcome to the comprehensive learning path for Rust backend development. This curriculum bridges the gap between the foundational theory taught in [The Rust Programming Language Book](https://doc.rust-lang.org/book/) and modern, production-grade backend engineering with Rust.

> **Status (2026-11-19):** Phase 1 and Phase 2 are complete — 11 book chapters fully covered, 39
> unit tests passing, Clippy clean. Open items from those phases: lifetime annotations (10.3) and
> `tests/` integration tests (11). Phase 3 (raw TCP/HTTP server) is next, then Phase 4.

---

## 🧭 Why Rust for Backend Engineering?

Traditional backend environments (Node.js, Python, Go, Java) make trade-offs:
- **Python / Node.js**: Fast development speed, but high memory footprint, slower execution speed, and dynamic typing bugs that slip into production.
- **Go / Java**: Statically typed with decent performance, but rely on garbage collection pauses and lack compile-time memory safety without runtime overhead.
- **Rust**: **Zero-cost abstractions**, **predictable latency (no garbage collector)**, **extremely low memory usage**, and **fearless concurrency** where data races are caught at compile time.

---

## 🗺️ Master Curriculum Overview

The curriculum is structured into four sequential phases:

```mermaid
flowchart LR
    subgraph P1 ["Phase 1: Language Core"]
        direction TB
        P1A["Ownership & Borrowing"]
        P1B["Structs & Methods"]
        P1C["Enums & Matching"]
        P1D["Collections"]
        P1E["Error Handling"]
    end

    subgraph P2 ["Phase 2: Abstractions & Concurrency"]
        direction TB
        P2A["Generics & Traits"]
        P2B["Iterators & Closures"]
        P2C["Smart Pointers"]
        P2D["Threads & Channels"]
    end

    subgraph P3 ["Phase 3: Network Fundamentals"]
        direction TB
        P3A["Ch 21: Raw Multithreaded TCP Server"]
        P3B["Manual ThreadPool & HTTP 1.1"]
    end

    subgraph P4 ["Phase 4: Modern Production Backend"]
        direction TB
        P4A["Async Rust & Tokio Runtime"]
        P4B["Axum Web Framework & Routing"]
        P4C["Serde JSON Serialization"]
        P4D["SQLx & SQLite Database CRUD"]
    end

    P1 --> P2 --> P3 --> P4
```

---

## 📚 Book Chapter vs Backend Mapping

| Rust Book Chapter | Key Concept | Why It Matters for Backend Engineering | Package in this Repo |
|---|---|---|---|
| **Ch 1–3** | Setup, Variables, Types, Loops | Basic syntax, Cargo builds | `hello_cargo`, `variables`, `datatypes`, `functions`, `conditions`, `loops`, `guessing_game` |
| **Ch 4** | **Ownership, References, Slices** | Eliminates use-after-free, double-free, and data races. Zero-copy header/string parsing. | `ownership`, `ownership_borrowing` |
| **Ch 5** | **Structs & Methods** | Modeling domain entities (User, Product), DTOs, and the Builder Pattern for DB/HTTP clients. | `structs_methods` |
| **Ch 6** | **Enums, Pattern Matching, `Option<T>`** | Null-safety, finite state machines (e.g. `OrderStatus::Pending`, `Shipped`), type-safe API responses. | `enums_matching` |
| **Ch 7** | **Modules, Packages & Crates** | Organizing clean backend codebases into `routes`, `models`, `services`, and `db`. | Built into project structures |
| **Ch 8** | **Collections (`Vec`, `HashMap`, `String`)** | In-memory key-value caching, payload batching, lookup tables. | `collections` |
| **Ch 9** | **Error Handling (`Result<T, E>`, `?`)** | Handling failed DB queries, invalid user input, and mapping to HTTP 4xx/5xx status codes. | `error_handling` |
| **Ch 10** | **Generics, Traits, Lifetimes** | Dependency injection (e.g. `trait UserRepository`), database abstractions, generic query handlers. | `traits_generics` *(Phase 2)* |
| **Ch 11** | Automated Testing | Unit testing services, mocking repository interfaces, integration test harnesses. | Used across all packages |
| **Ch 13** | **Iterators & Closures** | Functional pipelines for streaming data, filtering records, transforming DB result sets. | `iterators_closures` *(Phase 2)* |
| **Ch 15** | **Smart Pointers (`Box`, `Arc`, `Mutex`)** | Sharing connection pools (`Arc<Pool>`), state synchronization across worker threads. | `smart_pointers` *(Phase 2)* |
| **Ch 16** | **Concurrency (`std::thread`, `mpsc`)** | Background job queues, worker pools, multi-threaded pipelines. | `concurrency_threads` *(Phase 2)* |
| **Ch 17** | **Async / Await & Futures** | Non-blocking I/O: handling tens of thousands of concurrent network connections per machine. | `async_tokio` *(Phase 4)* |
| **Ch 21** | **Multithreaded Web Server** | Deep understanding of raw TCP sockets, HTTP 1.1 request lines, and thread pool worker mechanics. | `multithreaded_webserver` *(Phase 3)* |

---

## 🚀 The Backend Bridge: Beyond the Book

The official Rust book stops after teaching raw threads and standard library TCP listeners. In real-world backend production systems, you will build on top of these foundations using industry-standard libraries:

1. **Async Runtime**: [Tokio](https://tokio.rs/) — An asynchronous event loop runtime providing multi-threaded green-task scheduling, non-blocking network I/O, timers, and synchronization primitives.
2. **Web Framework**: [Axum](https://github.com/tokio-rs/axum) — An ergonomic, modular web framework built with Tokio and Tower, featuring type-safe request extractors, declarative routing, and composable middleware.
3. **Serialization**: [Serde](https://serde.rs/) & `serde_json` — Zero-copy and type-safe JSON serialization/deserialization for HTTP request payloads and responses.
4. **Database Driver**: [SQLx](https://github.com/launchbadge/sqlx) — A 100% safe, async, compile-time verified SQL toolkit with connection pooling and migrations.

---

## 🏃 Quick Start Commands

Run any package from the repository root:

```shell
# Phase 1 Packages
cargo run -p ownership_borrowing
cargo run -p structs_methods
cargo run -p enums_matching
cargo run -p collections
cargo run -p error_handling

# Run all unit tests across the entire workspace
cargo test --workspace

# Lint all code to production standards
cargo clippy --workspace --all-targets -- -D warnings
```
