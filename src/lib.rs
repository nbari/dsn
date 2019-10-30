use percent_encoding::percent_decode;
use std::{collections::HashMap, error::Error, fmt, str::Chars};

#[derive(Debug)]
pub enum ParseError {
    InvalidPort,
    InvalidDriver,
    MissingUsername,
    MissingProtocol,
    MissingAddress,
    MissingHost,
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
            ParseError::InvalidPort => "invalid port number",
            ParseError::InvalidDriver => "invalid driver",
            ParseError::MissingUsername => "missing username",
            ParseError::MissingProtocol => "missing protocol",
            ParseError::MissingAddress => "missing address",
            ParseError::MissingHost => "missing host",
            ParseError::MissingSocket => "missing unix domain socket",
        }
    }
}

/// driver://username:password@protocol(address)/dbname?param=value
#[derive(Debug, Default)]
pub struct DSN {
    pub driver: String,
    pub username: String,
    pub password: Option<String>,
    pub protocol: String,
    pub address: String,
    pub host: String,
    pub port: Option<u16>,
    pub database: Option<String>,
    pub socket: Option<String>,
    pub params: HashMap<String, String>,
}

pub fn parse(input: &str) -> Result<DSN, ParseError> {
    // create an empty DSN
    let mut dsn = DSN::default();

    // create an interator for input
    let chars = &mut input.chars();

    // <driver>://
    dsn.driver = get_driver(chars)?;

    // <username>:<password>@
    let (user, pass) = get_username_password(chars)?;
    dsn.username = user;
    if pass.len() > 0 {
        dsn.password = Some(pass);
    } else {
        dsn.password = None;
    }

    // protocol(
    dsn.protocol = get_protocol(chars)?;

    // address) <host:port|/path/to/socket>
    dsn.address = get_address(chars)?;

    //let (host, port, socket) = get_address(chars)?;
    //dsn.host = host;
    //dsn.socket = Some(socket);

    //if port.len() > 0 {
    //dsn.port = match port.parse::<u16>() {
    //Ok(n) => Some(n),
    //Err(_) => return Err(ParseError::InvalidPort),
    //}
    //} else if dsn.host != "unix" {
    //dsn.port = get_default_port(dsn.driver.as_str());
    //}

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
///    assert_eq!(dsn.username, "user");
///    assert_eq!(dsn.password.unwrap(), "o:o");
/// }
///```
fn get_username_password(chars: &mut Chars) -> Result<(String, String), ParseError> {
    let mut username = String::new();
    let mut password = String::new();
    // username
    while let Some(c) = chars.next() {
        match c {
            '@' => {
                if username.len() == 0 {
                    return Err(ParseError::MissingUsername);
                }
                break;
            }
            ':' => {
                if username.len() == 0 {
                    return Err(ParseError::MissingUsername);
                }
                break;
            }
            _ => username.push(c),
        }
    }
    // password
    while let Some(c) = chars.next() {
        match c {
            '@' => break,
            _ => password.push(c),
        }
    }
    Ok((
        percent_decode(username.as_bytes())
            .decode_utf8()
            .unwrap()
            .into(),
        percent_decode(password.as_bytes())
            .decode_utf8()
            .unwrap()
            .into(),
    ))
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

fn get_host_port_socket(chars: &mut Chars) -> Result<(String, String, String), ParseError> {
    let mut host = String::new();
    let mut port = String::new();
    let mut socket = String::new();
    let mut defined_port = false;

    // host
    while let Some(c) = chars.next() {
        match c {
            ':' | '/' => {
                if host.len() == 0 {
                    return Err(ParseError::MissingHost);
                }
                if c == '/' {
                    defined_port = false;
                }
                break;
            }
            _ => host.push(c),
        }
    }

    if defined_port {
        // port or socket
        while let Some(c) = chars.next() {
            match c {
                '/' => {
                    if host == "unix" && socket.len() == 0 {
                        return Err(ParseError::MissingSocket);
                    }
                    break;
                }
                _ => {
                    if host == "unix" {
                        socket.push(c);
                    } else {
                        port.push(c);
                    }
                }
            }
        }
    }

    Ok((host, port, socket))
}

pub fn get_default_port(scheme: &str) -> Option<u16> {
    match scheme {
        "mysql" => Some(3306),
        "pgsql" => Some(5432),
        "redis" => Some(6379),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    //#[test]
    //fn test_parse() {
    //let dsn = parse(r#"mysql://user:o%3Ao@tcp(localhost:3306)/database"#).unwrap();
    //println!("{:?}", dsn);
    //assert_eq!(dsn.driver, "mysql");
    //assert_eq!(dsn.username, "user");
    //assert_eq!(dsn.password.unwrap(), "o:o");
    //assert_eq!(dsn.protocol, "tcp");
    //assert_eq!(dsn.address, "localhost:3306");
    //assert_eq!(dsn.host, "host");
    //assert_eq!(dsn.port.unwrap(), 3306);
    //assert_eq!(dsn.database, None);
    //assert_eq!(dsn.socket.unwrap(), "");
    //}

    /*
    #[test]
    fn test_parse_password() {
    let dsn = parse(r#"mysql://user:pas':"'sword44444@host:port/database"#).unwrap();
    assert_eq!(dsn.password, r#"pas':"'sword44444"#);
    }
    */
}
