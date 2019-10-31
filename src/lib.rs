//! DSN (Data Source Name) parser
//!
//! [![crates.io](https://img.shields.io/crates/v/dsn.svg)](https://crates.io/crates/dsn)
//! [![Build Status](https://travis-ci.org/nbari/dsn.svg?branch=master)](https://travis-ci.org/nbari/dsn)
//!
//!
//! DSN format:
//!```text
//!<driver>://<username>:<password>@<protocol>(<address>)/<database>?param1=value1&...&paramN=valueN
//!```
//!
//! A DSN in its fullest form:
//!
//!```text
//!driver://username:password@protocol(address)/dbname?param=value
//!```
//! For protocol `TCP/UDP` address have the form `host:port`.
//!
//! For protocol `unix` (Unix domain sockets) the address is the absolute path to the socket.
//!
//! Connect to database on a non standard port:
//!
//!```text
//!pgsql://user:pass@tcp(localhost:5555)/dbname
//!```
//!
//! When using a Unix domain socket:
//!
//!```text
//!mysql://user@unix(/path/to/socket)/database
//!```

use percent_encoding::percent_decode;
use std::{collections::HashMap, error::Error, fmt, str::Chars};

#[derive(Debug)]
pub enum ParseError {
    InvalidDriver,
    InvalidPort,
    InvalidProtocol,
    InvalidSocket,
    MissingAddress,
    MissingHost,
    MissingProtocol,
    MissingSocket,
}

impl fmt::Display for ParseError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.description().fmt(fmt)
    }
}

impl Error for ParseError {
    fn description(&self) -> &str {
        match *self {
            ParseError::InvalidDriver => "invalid driver",
            ParseError::InvalidPort => "invalid port number",
            ParseError::InvalidProtocol => "invalid protocol",
            ParseError::InvalidSocket => "invalid socket",
            ParseError::MissingAddress => "missing address",
            ParseError::MissingHost => "missing host",
            ParseError::MissingProtocol => "missing protocol",
            ParseError::MissingSocket => "missing unix domain socket",
        }
    }
}

/// driver://username:password@protocol(address)/dbname?param=value
#[derive(Debug, Default)]
pub struct DSN {
    pub driver: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub protocol: String,
    pub address: String,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub database: Option<String>,
    pub socket: Option<String>,
    pub params: HashMap<String, String>,
}

/// Parse DSN
///
/// Example:
///
///```
///use dsn::parse;
///
///fn main() {
///    let dsn = parse(r#"mysql://user:o%3Ao@tcp(localhost:3306)/database"#).unwrap();
///    assert_eq!(dsn.driver, "mysql");
///    assert_eq!(dsn.username.unwrap(), "user");
///    assert_eq!(dsn.password.unwrap(), "o:o");
///    assert_eq!(dsn.protocol, "tcp");
///    assert_eq!(dsn.address, "localhost:3306");
///    assert_eq!(dsn.host.unwrap(), "localhost");
///    assert_eq!(dsn.port.unwrap(), 3306);
///    assert_eq!(dsn.database.unwrap(), "database");
///    assert_eq!(dsn.socket, None);
///}
///```
pub fn parse(input: &str) -> Result<DSN, ParseError> {
    // create an empty DSN
    let mut dsn = DSN::default();

    // create an interator for input
    let chars = &mut input.chars();

    // <driver>://
    dsn.driver = get_driver(chars)?;

    // <username>:<password>@
    let (user, pass) = get_username_password(chars)?;
    if user.len() > 0 {
        dsn.username = Some(user);
    } else {
        dsn.username = None;
    }
    if pass.len() > 0 {
        dsn.password = Some(pass);
    } else {
        dsn.password = None;
    }

    // protocol(
    dsn.protocol = get_protocol(chars)?;

    // address) <host:port|/path/to/socket>
    dsn.address = get_address(chars)?;

    if dsn.protocol == "unix" {
        if !dsn.address.starts_with("/") {
            return Err(ParseError::InvalidSocket);
        }
        dsn.socket = Some(dsn.address.clone())
    } else if dsn.protocol == "tcp" {
        let (host, port) = get_host_port(dsn.address.clone())?;
        dsn.host = Some(host);

        if port.len() > 0 {
            dsn.port = match port.parse::<u16>() {
                Ok(n) => Some(n),
                Err(_) => return Err(ParseError::InvalidPort),
            }
        }
    }

    // /<database>?
    let database = get_database(chars)?;
    if database.len() > 0 {
        dsn.database = Some(database);
    } else {
        dsn.database = None;
    }

    Ok(dsn)
}

/// Example:
///
///```
///use dsn::parse;
///
///fn main() {
///    let dsn = parse(r#"mysql://user:o%3Ao@tcp(localhost:3306)/database"#).unwrap();
///    assert_eq!(dsn.driver, "mysql");
/// }
///```
fn get_driver(chars: &mut Chars) -> Result<String, ParseError> {
    let mut driver = String::new();
    while let Some(c) = chars.next() {
        if c == ':' {
            if chars.next() == Some('/') && chars.next() == Some('/') {
                break;
            }
            return Err(ParseError::InvalidDriver);
        }
        driver.push(c);
    }
    Ok(driver)
}

/// Example:
///
///```
///use dsn::parse;
///
///fn main() {
///    let dsn = parse(r#"mysql://user:o%3Ao@tcp(localhost:3306)/database"#).unwrap();
///    assert_eq!(dsn.username.unwrap(), "user");
///    assert_eq!(dsn.password.unwrap(), "o:o");
/// }
///```
fn get_username_password(chars: &mut Chars) -> Result<(String, String), ParseError> {
    let mut username = String::new();
    let mut password = String::new();
    let mut has_password = true;

    // username
    while let Some(c) = chars.next() {
        match c {
            '@' => {
                has_password = false;
                break;
            }
            ':' => {
                break;
            }
            _ => username.push(c),
        }
    }

    username = percent_decode(username.as_bytes())
        .decode_utf8()
        .unwrap()
        .into();

    // password
    if has_password {
        while let Some(c) = chars.next() {
            match c {
                '@' => break,
                _ => password.push(c),
            }
        }
        password = percent_decode(password.as_bytes())
            .decode_utf8()
            .unwrap()
            .into();
    }
    Ok((username, password))
}

/// Example:
///
///```
///use dsn::parse;
///
///fn main() {
///    let dsn = parse(r#"mysql://user:o%3Ao@tcp(localhost:3306)/database"#).unwrap();
///    assert_eq!(dsn.protocol, "tcp");
/// }
///```
fn get_protocol(chars: &mut Chars) -> Result<String, ParseError> {
    let mut protocol = String::new();
    while let Some(c) = chars.next() {
        match c {
            '(' => {
                if protocol.len() == 0 {
                    return Err(ParseError::MissingProtocol);
                }
                break;
            }
            _ => protocol.push(c),
        }
    }
    Ok(protocol)
}

/// Example:
///
///```
///use dsn::parse;
///
///fn main() {
///    let dsn = parse(r#"mysql://user:o%3Ao@tcp(localhost:3306)/database"#).unwrap();
///    assert_eq!(dsn.address, "localhost:3306");
/// }
///```
fn get_address(chars: &mut Chars) -> Result<String, ParseError> {
    let mut address = String::new();
    while let Some(c) = chars.next() {
        match c {
            ')' => {
                if address.len() == 0 {
                    return Err(ParseError::MissingAddress);
                }
                break;
            }
            _ => address.push(c),
        }
    }
    Ok(address)
}

/// Example:
///
///```
///use dsn::parse;
///
///fn main() {
///    let dsn = parse(r#"mysql://user:o%3Ao@tcp(localhost:3306)/database"#).unwrap();
///    assert_eq!(dsn.host.unwrap(), "localhost");
///    assert_eq!(dsn.port.unwrap(), 3306);
/// }
///```
fn get_host_port(address: String) -> Result<(String, String), ParseError> {
    let mut host = String::new();
    let mut chars = address.chars();

    // host
    while let Some(c) = chars.next() {
        match c {
            ':' => {
                if host.len() == 0 {
                    return Err(ParseError::MissingHost);
                }
                break;
            }
            _ => host.push(c),
        }
    }

    // port
    let port = chars.as_str();

    Ok((host, port.into()))
}

/// Example:
///
///```
///use dsn::parse;
///
///fn main() {
///    let dsn = parse(r#"mysql://user:o%3Ao@tcp(localhost:3306)/database"#).unwrap();
///    assert_eq!(dsn.database.unwrap(), "database");
/// }
///```
fn get_database(chars: &mut Chars) -> Result<String, ParseError> {
    let mut database = String::new();
    while let Some(c) = chars.next() {
        match c {
            '/' => {
                if database.len() == 0 {
                    continue;
                }
            }
            '?' => break,
            _ => database.push(c),
        }
    }
    Ok(database)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_password() {
        let dsn = parse(r#"mysql://user:pas':"'sword44444@host:port/database"#).unwrap();
        assert_eq!(dsn.password.unwrap(), r#"pas':"'sword44444"#);
    }
}
