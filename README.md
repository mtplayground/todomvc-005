# TodoMVC - Rust/Leptos/Axum

A full-stack TodoMVC implementation built with Rust using:
- **Leptos 0.7** - Full-stack reactive web framework with SSR
- **Axum 0.7** - Fast async web server
- **SQLx 0.8** - Async SQLite database with migrations
- **Tokio** - Async runtime

## Features

- Create, read, update, delete todos
- Toggle individual todos complete/incomplete
- Toggle all todos at once
- Inline editing with double-click
- Filter todos: All / Active / Completed
- Clear all completed todos
- Persistent storage with SQLite
- Server-side rendering (SSR) with optional client-side hydration

## Requirements

- Rust (nightly toolchain configured in rust-toolchain.toml or via rustup)
- `cargo`

## Build & Run

```bash
# Set the database URL for SQLx compile-time checks
export DATABASE_URL=sqlite:todos.db
touch todos.db

# Build the server binary
cargo build --features ssr --release

# Run the server
./target/release/todomvc
```

The server will start on `http://0.0.0.0:8080`.

## Development

```bash
export DATABASE_URL=sqlite:todos.db
touch todos.db

# Development build
cargo build --features ssr

# Run tests
cargo test --features ssr
```

## Database

SQLite database is created automatically on first run. Migrations are applied automatically at startup.

Migration: `migrations/0001_create_todos.sql` - Creates the `todos` table with:
- `id` - Primary key (autoincrement)
- `title` - Todo text
- `completed` - Boolean completion status
- `display_order` - Order for display

## Architecture

```
src/
  main.rs    - Axum server setup, database pool, route configuration
  lib.rs     - Library entry point, hydrate function for WASM
  app.rs     - All Leptos components and server functions

migrations/
  0001_create_todos.sql - Database schema

tests/
  integration.rs - SQLite integration tests

style/
  main.scss  - TodoMVC CSS styles
```

## Server Functions

All data operations are implemented as Leptos server functions:
- `get_todos` - Fetch all todos ordered by display_order
- `add_todo` - Create a new todo with auto-incrementing order
- `update_todo` - Update todo title (deletes if empty)
- `delete_todo` - Remove a todo by ID
- `toggle_todo` - Toggle completion status
- `toggle_all` - Set all todos to a given completion state
- `clear_completed` - Delete all completed todos
