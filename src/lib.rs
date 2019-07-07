use std::{error::Error, fmt};

#[derive(Debug, Default)]
pub struct DSN {
    driver: &'static str,
    username: &'static str,
    password: &'static str,
    host: &'static str,
    port: u16,
    database: &'static str,
    socket: &'static str,
}

#[derive(Debug)]
pub enum ParseError {
    InvalidPort,
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
        }
    }
}

pub fn parse(input: &str) -> Result<DSN, ParseError> {
    for c in input.chars() {
        match c {
            ':' => println!("push to buffer"),
            _ => println!("{}", c),
        }
    }
    let dsn = DSN::default();
    Ok(dsn)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

pub fn default_port(scheme: &str) -> Option<u16> {
    match scheme {
        "mysql" => Some(3306),
        "pgsql" => Some(5432),
        "redis" => Some(6379),
        _ => None,
    }
}
