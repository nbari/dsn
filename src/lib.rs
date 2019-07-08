use std::{error::Error, fmt};

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
        }
    }
}

pub fn parse(input: &str) -> Result<DSN, ParseError> {
    let mut dsn = DSN::default();

    let mut chars = get_driver(input.chars())?;

    loop {
        let c = chars.next();
        if c == Some(':') {
            if chars.next() == Some('/') && chars.next() == Some('/') {
                break;
            }
            return Err(ParseError::InvalidDriver);
        }
        dsn.driver.push(c.unwrap());
    }
    println!("{:?}", dsn);

    Ok(dsn)
}

fn get_driver(chars: std::str::Chars) -> Result<std::str::Chars, ParseError> {
    let x = chars;
    for c in x {
        println!("{}", c);
    }
    Ok(ck)
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
        let dsn = parse("mysql://user:password@host:port/database").unwrap();
        assert_eq!(dsn.driver, "mysql");
    }
}
