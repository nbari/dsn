use std::{error::Error, fmt};
use url::percent_encoding::percent_decode;

#[derive(Debug, Default)]
pub struct DSN {
    driver: String,
    username: String,
    password: String,
    host: String,
    port: Option<u16>,
    database: String,
    socket: String,
}

#[derive(Debug)]
pub enum ParseError {
    InvalidPort,
    InvalidDriver,
    MissingUsername,
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
            ParseError::MissingHost => "missing host",
            ParseError::MissingSocket => "missing unix domain socket",
        }
    }
}

pub fn parse(input: &str) -> Result<DSN, ParseError> {
    let mut dsn = DSN::default();

    let chars = &mut input.chars();

    dsn.driver = get_driver(chars)?;
    let (user, pass) = get_username_password(chars)?;
    dsn.username = user;
    dsn.password = pass;
    let (host, port, socket) = get_host_port_socket(chars)?;
    dsn.host = host;
    dsn.socket = socket;

    if port.len() > 0 {
        dsn.port = match port.parse::<u16>() {
            Ok(n) => Some(n),
            Err(_) => return Err(ParseError::InvalidPort),
        }
    } else if dsn.host != "unix" {
        dsn.port = get_default_port(dsn.driver.as_str());
    }

    Ok(dsn)
}

fn get_driver(chars: &mut std::str::Chars) -> Result<String, ParseError> {
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

fn get_username_password(chars: &mut std::str::Chars) -> Result<(String, String), ParseError> {
    let mut username = String::new();
    let mut password = String::new();
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

fn get_host_port_socket(
    chars: &mut std::str::Chars,
) -> Result<(String, String, String), ParseError> {
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

    #[test]
    fn test_parse() {
        let dsn = parse(r#"mysql://user:o%3Ao@host/database"#).unwrap();
        println!("{:?}", dsn);
        assert_eq!(dsn.driver, "mysql");
        assert_eq!(dsn.username, "user");
        assert_eq!(dsn.password, "o:o");
        assert_eq!(dsn.host, "host");
        assert_eq!(dsn.port, Some(3306));
        assert_eq!(dsn.database, "");
        assert_eq!(dsn.socket, "");
    }

    /*
    #[test]
    fn test_parse_password() {
        let dsn = parse(r#"mysql://user:pas':"'sword44444@host:port/database"#).unwrap();
        assert_eq!(dsn.password, r#"pas':"'sword44444"#);
    }
    */
}
