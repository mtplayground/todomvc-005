# TodoMVC - Leptos + Axum + SQLite

A full-stack TodoMVC application built with Leptos (full-stack Rust web framework), Axum HTTP server, and SQLite via SQLx.

## Features

- Create, read, update, delete todos
- Toggle individual todos or all at once
- Filter by All / Active / Completed
- Clear all completed todos
- Inline editing (double-click)
- Persistent storage via SQLite
- Server-side rendering with client-side hydration

## Prerequisites

- Rust nightly (see `rust-toolchain.toml`)
- `cargo-leptos` CLI tool
- `wasm32-unknown-unknown` target

## Setup

Install cargo-leptos:

```bash
cargo install cargo-leptos
```

## Build

```bash
cargo leptos build --release
```

Or for development:

```bash
cargo leptos watch
```

## Run

```bash
DATABASE_URL=sqlite:todos.db ./target/release/todomvc
```

The server listens on `0.0.0.0:8080` by default.

## Environment Variables

- `DATABASE_URL` - SQLite connection string (default: `sqlite:todos.db`)

## Development

The app uses:

- `leptos` 0.6 for reactive UI components
- `leptos_axum` for SSR integration
- `axum` 0.7 as the HTTP server
- `sqlx` 0.7 with SQLite for persistence
- `leptos_router` for client-side routing

## Project Structure

```
src/
  main.rs    - Axum server setup, database pool init
  lib.rs     - WASM hydration entry point
  app.rs     - All UI components and server functions
style/
  main.css   - TodoMVC CSS styles
migrations/
  0001_create_todos.sql - Database schema
tests/
  integration.rs - Integration tests
```
