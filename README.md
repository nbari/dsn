# DSN - Data Source Name Parser & Builder

[![crates.io](https://img.shields.io/crates/v/dsn.svg)](https://crates.io/crates/dsn)
[![Test & Build](https://github.com/nbari/dsn/actions/workflows/build.yml/badge.svg)](https://github.com/nbari/dsn/actions/workflows/build.yml)
[![codecov](https://codecov.io/gh/nbari/dsn/graph/badge.svg?token=72ZRLMJGQD)](https://codecov.io/gh/nbari/dsn)
[![docs](https://docs.rs/dsn/badge.svg)](https://docs.rs/dsn)
[![License](https://img.shields.io/badge/license-BSD--3--Clause-blue.svg)](LICENSE)

A lightweight, fast, and type-safe Rust library for parsing and building Data Source Name (DSN) strings for databases like MySQL, PostgreSQL, Redis, and more.

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [DSN Format](#dsn-format)
- [Quick Start](#quick-start)
  - [Parsing DSN Strings](#parsing-dsn-strings)
  - [Building DSN Strings](#building-dsn-strings)
  - [Converting Back to String](#converting-back-to-string)
  - [Error Handling](#error-handling)
- [Database-Specific Builders](#database-specific-builders)
- [Examples](#examples)
- [Real-World Integration](#real-world-integration)
- [Contributing](#contributing)
- [License](#license)

## Features

- **Parse DSNs**: Parse existing DSN strings into structured, type-safe data structures
- **Build DSNs**: Construct DSN strings programmatically with a fluent builder API
- **Percent Encoding**: Automatic percent-encoding for special characters in credentials
- **Database Support**: Pre-configured builders for MySQL, PostgreSQL, Redis, and MariaDB
- **Protocol Support**: TCP, Unix sockets, and file paths
- **Type Safety**: Comprehensive error handling with descriptive error types
- **Zero Dependencies**: Only depends on `percent-encoding`
- **Edition 2024**: Built with the latest Rust edition

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
dsn = "1.1"
```

## DSN Format

The general DSN format is:

```
<driver>://<username>:<password>@<protocol>(<address>)/<database>?param1=value1&...&paramN=valueN
```

A complete DSN example:

```
mysql://username:password@tcp(host:port)/dbname?charset=utf8mb4
```

### Protocol Support

The address format changes depending on the protocol:

**TCP/UDP** - `host:port` format:
```
postgresql://user:pass@tcp(localhost:5432)/dbname
```

**Unix Domain Sockets** - absolute path to socket:
```
mysql://user@unix(/var/run/mysqld/mysqld.sock)/database
```

**File Paths** (SQLite) - absolute file path:
```
sqlite://@file(/full/unix/path/to/file.db)
```

### Percent Encoding

Special characters in usernames and passwords are automatically percent-encoded when using the builder API. For parsing, you can manually encode credentials:

```bash
# Using jq
echo -n "p@ss:word!" | jq -s -R -r @uri

# Using xxd
echo -n "p@ss:word!" | xxd -p | sed 's/../%&/g'
```

Result:
```
mysql://root:p%40ss%3Aword%21@tcp(localhost:3306)/test
```

## Quick Start

### Parsing DSN Strings

```rust
use dsn::parse;

let dsn = parse("mysql://user:pass@tcp(localhost:3306)/database?charset=utf8mb4")?;

println!("Driver: {}", dsn.driver);
println!("Host: {}", dsn.host.unwrap());
println!("Port: {}", dsn.port.unwrap());
println!("Database: {}", dsn.database.unwrap());
println!("Charset: {}", dsn.params.get("charset").unwrap());
```

### Building DSN Strings

```rust
use dsn::DSNBuilder;

// MySQL
let mysql = DSNBuilder::mysql()
    .username("root")
    .password("secret")
    .host("localhost")
    .database("myapp")
    .param("charset", "utf8mb4")
    .build();

println!("{}", mysql);
// Output: mysql://root:secret@tcp(localhost:3306)/myapp?charset=utf8mb4

// PostgreSQL with SSL
let postgres = DSNBuilder::postgres()
    .username("postgres")
    .password("admin")
    .host("db.example.com")
    .database("production")
    .param("sslmode", "require")
    .build();

// PostgreSQL with SSL disabled (development)
let postgres_dev = DSNBuilder::postgres()
    .username("dev")
    .password("dev123")
    .host("localhost")
    .database("dev_db")
    .param("sslmode", "disable")
    .build();

// Redis
let redis = DSNBuilder::redis()
    .host("cache.example.com")
    .password("redis-pass")
    .database("0")
    .build();

// Unix Socket
let socket_dsn = DSNBuilder::mysql()
    .username("app")
    .socket("/var/run/mysqld/mysqld.sock")
    .database("appdb")
    .build();
```

### Converting Back to String

DSN structs implement `Display`, so you can convert them back to strings:

```rust
use dsn::parse;

let original = "mysql://root:pass@tcp(localhost:3306)/db";
let dsn = parse(original)?;

// Convert back to string
let dsn_string = dsn.to_string();
// or
let dsn_string = format!("{}", dsn);
```

### Error Handling

The library provides descriptive error types:

```rust
use dsn::{parse, ParseError};

match parse("invalid://dsn") {
    Ok(dsn) => println!("Parsed: {}", dsn.driver),
    Err(ParseError::InvalidDriver) => eprintln!("Driver is invalid"),
    Err(ParseError::InvalidPort) => eprintln!("Port number is invalid"),
    Err(ParseError::MissingHost) => eprintln!("Host is required"),
    Err(e) => eprintln!("Parse error: {}", e),
}
```

Available error types:
- `InvalidDriver` - Driver name is invalid or missing
- `InvalidProtocol` - Protocol is invalid
- `InvalidSocket` - Unix socket path is invalid
- `InvalidPath` - File path is not absolute
- `InvalidPort` - Port number is invalid or out of range
- `InvalidParams` - Query parameters are malformed
- `MissingAddress` - Address is missing after protocol
- `MissingHost` - Host is missing in address
- `Utf8Error` - UTF-8 decoding error in credentials

## Database-Specific Builders

Pre-configured builders with sensible defaults:

| Builder | Driver | Default Port | Use Case |
|---------|--------|--------------|----------|
| `DSNBuilder::mysql()` | mysql | 3306 | MySQL databases |
| `DSNBuilder::postgres()` | postgres | 5432 | PostgreSQL databases |
| `DSNBuilder::redis()` | redis | 6379 | Redis cache/store |
| `DSNBuilder::mariadb()` | mariadb | 3306 | MariaDB databases |

## Examples

See the [examples](examples/) directory for comprehensive examples:

```bash
# General DSN building examples
cargo run --example builder

# PostgreSQL SSL mode examples
cargo run --example postgres_ssl
```

## Real-World Integration

### MySQL

Using with the [mysql](https://crates.io/crates/mysql) crate:

```rust
use dsn::parse;

let dsn = parse("mysql://user:password@tcp(db.example.com)/mydb?tls=skip-verify")?;

let mut opts = mysql::OptsBuilder::new();
opts.user(dsn.username);
opts.pass(dsn.password);
opts.ip_or_hostname(dsn.host);
if let Some(port) = dsn.port {
    opts.tcp_port(port);
}
opts.db_name(dsn.database);

let pool = mysql::Pool::new(opts)?;
```

### PostgreSQL

Using with the [postgres](https://crates.io/crates/postgres) crate:

```rust
use dsn::DSNBuilder;

let dsn = DSNBuilder::postgres()
    .username("postgres")
    .password("secret")
    .host("localhost")
    .database("mydb")
    .param("sslmode", "require")
    .build();

let client = postgres::Client::connect(&dsn.to_string(), postgres::NoTls)?;
```

### Redis

Using with the [redis](https://crates.io/crates/redis) crate:

```rust
use dsn::DSNBuilder;

let dsn = DSNBuilder::redis()
    .host("localhost")
    .password("secret")
    .database("0")
    .build();

let client = redis::Client::open(dsn.to_string())?;
```

## Supported Rust Versions

This crate requires Rust 1.85 or later due to the use of Edition 2024.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

BSD-3-Clause - See [LICENSE](LICENSE) for details.

## Links

- [Documentation](https://docs.rs/dsn)
- [Crates.io](https://crates.io/crates/dsn)
- [Repository](https://github.com/nbari/dsn)
- [Issue Tracker](https://github.com/nbari/dsn/issues)
