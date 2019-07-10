use std::{error::Error, fmt};
use url::percent_encoding::percent_decode;

#[derive(Debug, Default)]
pub struct DSN {
    driver: String,
    username: String,
    password: String,
    host: String,
    port: u16,
    database: String,
    socket: String,
}

#[derive(Debug)]
pub enum ParseError {
    InvalidPort,
    InvalidDriver,
    MissingUsername,
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
            _ => (),
        }
        username.push(c);
    }
    while let Some(c) = chars.next() {
        match c {
            '@' => break,
            _ => (),
        }
        password.push(c);
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

pub fn default_port(scheme: &str) -> Option<u16> {
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
        // let dsn = parse(r#"mysql://user:pas':"'sword44444@host:port/database"#).unwrap();
        let dsn = parse(r#"mysql://user:pas':"'sword4o%3Ao@host:port/database"#).unwrap();
        println!("{:?}", dsn);
        assert_eq!(dsn.driver, "mysql");
    }

    /*
    #[test]
    fn test_parse_password() {
        let dsn = parse(r#"mysql://user:pas':"'sword44444@host:port/database"#).unwrap();
        assert_eq!(dsn.password, r#"pas':"'sword44444"#);
    }
    */
}
