# AGENTS.md - Rust Learning Repository

## Project Overview

This is a Rust learning repository containing multiple modules, each focusing on different aspects of Rust programming. The repository is structured as a collection of mini-projects/demos covering core Rust concepts.

## Directory Structure

The repository contains the following main modules:

- `borrow/` - Rust borrowing and reference examples
- `lifetime/` - Rust lifetime examples
- `memory/` - Memory management concepts
- `filename-cleaner/` - A utility for cleaning filenames
- `ownership/` - Ownership and move semantics examples
- `queryer/` - A query processing system with SQL parsing and HTTP capabilities
- `type_system/` - Type system concepts
- `traits/` - Rust trait examples
- `thumbor/` - A Rust implementation of the Thumbor image processing service
- `src/` - Root source directory with basic hello world example

## Development Commands

### Building and Running

Each module can be built and run independently using Cargo:

```bash
# Build a specific module
cd borrow
cargo build

# Run a specific binary in a module
cd borrow
cargo run --bin heap_reference_stack

# Run tests for a module
cd borrow
cargo test

# Run all binaries/examples in a module
cd queryer
cargo run --example covid
cargo run --example dialect
```

### Common Cargo Commands

```bash
# Check code
cargo check

# Build release version
cargo build --release

# Format code
cargo fmt

# Run clippy linter
cargo clint
```

## Module-Specific Details

### Borrow Module (`borrow/`)
- Multiple binary targets for different borrowing concepts
- Focus on heap references and stack references
- Key binaries: `heap_reference_stack`, `heap_reference_stack_outlive`

### Queryer Module (`queryer/`)
- A sophisticated query processing system
- Dependencies: sqlparser, polars, reqwest, tokio, tracing
- Examples: `covid`, `dialect`
- Features SQL parsing, DataFrame operations, HTTP requests

### Thumbor Module (`thumbor/`)
- Image processing service implementation
- Uses Protocol Buffers for communication (build.rs generates pb code)
- Multiple server implementations: `server1`, `server2`
- Dependencies: axum, image, photon-rs, prost, tokio
- Key features: image resizing, cropping, filters, watermarks

### Other Modules
- `lifetime/`, `ownership/`, `memory/` - Core Rust concept examples
- `filename-cleaner/` - Practical utility project
- `type_system/`, `traits/` - Advanced Rust feature examples

## Code Conventions and Patterns

### Organization
- Each module is self-contained with its own `Cargo.toml`
- Modules use `mod` declarations to organize code
- Binary targets are defined in `Cargo.toml` with explicit paths

### Dependencies
- Minimal dependencies in simple concept modules
- Rich dependencies in practical application modules (queryer, thumbor)
- Comments in Chinese explaining dependencies (seen in queryer/Cargo.toml)

### Error Handling
- Uses `anyhow` for error handling in application modules
- Basic error handling in concept modules
- StatusCode returns in web service (thumbor/server1.rs)

### Asynchronous Programming
- Uses `tokio` for async operations
- `#[tokio::main]` macro for async main functions
- Async/await patterns in HTTP services

### Web Development
- Uses `axum` web framework
- Path extraction with `Path` extractor
- Router-based routing
- JSON handling with `serde`

## Build System

### Protocol Buffers
- `thumbor/` module uses Protocol Buffers
- Build script (`build.rs`) generates Rust code from `.proto` files
- Generated code goes to `src/pb/` directory

### Binary Targets
- Multiple binary targets per module where applicable
- Each target focuses on a specific concept or use case

## Testing

- Each module can have its own tests
- Run tests per module: `cargo test`
- Examples can be run: `cargo run --example <name>`

## Gotchas and Important Notes

1. **Module Independence**: Each directory is a separate Rust crate and should be developed/tested independently
2. **No Root Workspace**: This is not a Cargo workspace, each module is standalone
3. **Chinese Comments**: Some dependencies are documented in Chinese, indicating this may be used for Chinese-speaking developers
4. **Proto Code Generation**: The thumbor module requires running `cargo build` to generate Protocol Buffer code before development
5. **Multiple Binaries**: Some modules have multiple binary targets - specify which to run with `--bin <name>`
6. **No Global Scripts**: No Makefile or shell scripts in root - use Cargo commands per module
7. **Example Code**: Many modules contain example/demo code rather than production-ready applications
8. **Learning Focus**: The repository appears designed for learning Rust concepts rather than building production systems

## Working in This Repository

1. Navigate to the specific module directory first
2. Use Cargo commands for building/running/testing that module
3. Read the `Cargo.toml` to understand available binaries and dependencies
4. Check module structure to understand the concept being demonstrated
5. For web services (thumbor), run the server and test endpoints
6. For Protocol Buffer usage, run `cargo build` first to generate code