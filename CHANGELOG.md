# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.2.0] - 2025-11-14

### Added
- **DSN Builder API**: New fluent API for constructing DSN strings programmatically
  - `DSN::builder()` method to create a new builder
  - `DSNBuilder` struct with builder pattern methods
  - Database-specific constructors: `mysql()`, `postgres()`, `redis()`, `mariadb()`
  - Method chaining for host, port, username, password, database, and parameters
  - Unix socket support with `.socket()` method
- **Display Implementation**: `DSN` now implements `Display` trait
  - Convert DSN structs back to strings with `.to_string()` or `format!("{}", dsn)`
  - Automatic percent-encoding of credentials
- **Comprehensive Documentation**: All public APIs now have detailed documentation
  - Struct-level documentation with examples
  - Function-level documentation with arguments, returns, and error descriptions
  - Enum variant documentation for `ParseError`
- **Examples**: Added example programs demonstrating usage
  - `examples/builder.rs` - General DSN building examples (10 scenarios)
  - `examples/postgres_ssl.rs` - PostgreSQL SSL mode examples with parsing and validation

### Changed
- **Edition 2024**: Upgraded from Rust Edition 2021 to Edition 2024
  - Minimum Rust version: 1.85
  - Updated code to meet Edition 2024 requirements
- **Code Quality Improvements**:
  - Enabled strict Clippy lints (`clippy::all`, `clippy::nursery`, `clippy::pedantic`)
  - Fixed all clippy warnings and applied idiomatic Rust patterns
  - Improved error handling with `map_err` pattern
  - Optimized string handling and performance
- **CI/CD Enhancements**:
  - Added `rust-cache` for faster builds (Swatinem/rust-cache@v2)
  - Added nightly Rust testing to CI
  - Improved coverage reporting with `cargo-llvm-cov`
  - Added artifact uploads for build verification
  - ARM64 support for Linux and macOS builds
  - Updated to `actions/checkout@v4`

### Fixed
- Removed `#![warn]` attributes from source code, moved to `Cargo.toml` lints configuration
- Fixed ownership issues in builder pattern
- Improved percent-encoding/decoding for special characters

### Documentation
- Enhanced README with:
  - Better structured sections
  - Installation instructions
  - Protocol support details
  - Real-world integration examples for MySQL, PostgreSQL, and Redis
  - Database-specific builders table
  - Links to documentation and resources
- Added comprehensive inline documentation for all public APIs

## [1.1.2] - Previous Release

### Features
- Parse DSN strings into structured `DSN` objects
- Support for multiple protocols: TCP, Unix sockets, file paths
- Percent-decoding for usernames and passwords
- Query parameter parsing
- Error handling with descriptive `ParseError` types

### Supported DSN Format
```
<driver>://<username>:<password>@<protocol>(<address>)/<database>?param1=value1
```

[Unreleased]: https://github.com/nbari/dsn/compare/v1.2.0...HEAD
[1.2.0]: https://github.com/nbari/dsn/compare/v1.1.2...v1.2.0
[1.1.2]: https://github.com/nbari/dsn/releases/tag/v1.1.2
