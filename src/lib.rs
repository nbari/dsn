//!DSN (Data Source Name) parser
//!
//!DSN format:
//!```text
//!<driver>://<username>:<password>@<protocol>(<address>)/<database>?param1=value1&...&paramN=valueN
//!```
//!
//!A DSN in its fullest form:
//!
//!```text
//!driver://username:password@protocol(address)/dbname?param=value
//!```
//!
//!The address changes depending on the protocol
//!
//!For `TCP/UDP` address have the form `host:port`, example:
//!
//!```text
//!pgsql://user:pass@tcp(localhost:5555)/dbname
//!```
//!
//!For protocol `unix` (Unix domain sockets) the address is the absolute path to the socket, for example:
//!
//!```text
//!mysql://user@unix(/path/to/socket)/database
//!```
//!
//!For protocol `file` (sqlite) use the absolute path as the address, example:
//!
//!```text
//!sqlite://@file(/full/unix/path/to/file.db)
//!```
//!# percent-encode
//!
//!Percent-encode username and password with characters like `@`, for example if password is:
//!
//!```text
//!sop@s
//!
//!!A4T@hh'cUj7LXXvk"
//!```
//!
//!From the command line you can encode it with:
//!
//!```text
//!echo -n "sop@s" | jq -s -R -r @uri
//!```
//!or
//!
//!```text
//!echo -n "\!A4T@hh'cUj7LXXvk\"" | xxd -p |sed 's/../%&/g'
//!```
//!
//!Then you can build the dsn:
//!
//!```text
//!mysql://root:sop%40s@tcp(10.0.0.1:3306)/test
//!```
//!or
//!
//!```text
//!mysql://root:%21%41%34%54%40%68%68%27%63%55%6a%37%4c%58%58%76%6b%22@tcp(10.0.0.1:3306)/test
//!```
use percent_encoding::percent_decode;
use std::{collections::BTreeMap, error::Error, fmt, str::Chars};

#[derive(Debug)]
pub enum ParseError {
    InvalidDriver,
    InvalidParams,
    InvalidPath,
    InvalidPort,
    InvalidProtocol,
    InvalidSocket,
    MissingAddress,
    MissingHost,
    MissingProtocol,
    MissingSocket,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::InvalidDriver => write!(f, "invalid driver"),
            Self::InvalidParams => write!(f, "invalid params"),
            Self::InvalidPath => write!(f, "invalid absolute path"),
            Self::InvalidPort => write!(f, "invalid port number"),
            Self::InvalidProtocol => write!(f, "invalid protocol"),
            Self::InvalidSocket => write!(f, "invalid socket"),
            Self::MissingAddress => write!(f, "missing address"),
            Self::MissingHost => write!(f, "missing host"),
            Self::MissingProtocol => write!(f, "missing protocol"),
            Self::MissingSocket => write!(f, "missing unix domain socket"),
        }
    }
}

impl Error for ParseError {}

/// DSN format: `driver://username:password@protocol(address)/dbname?param=value`
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
    pub params: BTreeMap<String, String>,
}

/// Parse DSN
///
/// Example:
///
///```
///use dsn::parse;
///
///let dsn = parse(r#"mysql://user:o%3Ao@tcp(localhost:3306)/database?charset=utf8"#).unwrap();
///assert_eq!(dsn.driver, "mysql");
///assert_eq!(dsn.username.unwrap(), "user");
///assert_eq!(dsn.password.unwrap(), "o:o");
///assert_eq!(dsn.protocol, "tcp");
///assert_eq!(dsn.address, "localhost:3306");
///assert_eq!(dsn.host.unwrap(), "localhost");
///assert_eq!(dsn.port.unwrap(), 3306);
///assert_eq!(dsn.database.unwrap(), "database");
///assert_eq!(dsn.socket, None);
///assert!(!dsn.params.is_empty());
///assert_eq!(dsn.params.get("charset").unwrap(), ("utf8"));
///```
/// # Errors
/// [`ParseError`](enum.ParseError.html)
pub fn parse(input: &str) -> Result<DSN, ParseError> {
    // create an empty DSN
    let mut dsn = DSN::default();

    // create an interator for input
    let chars = &mut input.chars();

    // <driver>://
    dsn.driver = get_driver(chars)?;

    // <username>:<password>@
    let (user, pass) = get_username_password(chars)?;
    if !user.is_empty() {
        dsn.username = Some(user);
    }
    if !pass.is_empty() {
        dsn.password = Some(pass);
    }

    // protocol(
    dsn.protocol = get_protocol(chars)?;

    // address) <host:port|/path/to/socket>
    dsn.address = get_address(chars)?;

    match dsn.protocol.as_ref() {
        "unix" => {
            if !dsn.address.starts_with('/') {
                return Err(ParseError::InvalidSocket);
            }
            dsn.socket = Some(dsn.address.clone())
        }
        "file" => {
            if !dsn.address.starts_with('/') {
                return Err(ParseError::InvalidPath);
            }
        }
        _ => {
            let (host, port) = get_host_port(&dsn.address)?;
            dsn.host = Some(host);

            if !port.is_empty() {
                dsn.port = match port.parse::<u16>() {
                    Ok(n) => Some(n),
                    Err(_) => return Err(ParseError::InvalidPort),
                }
            }
        }
    }

    // /<database>?
    let database = get_database(chars)?;
    if !database.is_empty() {
        dsn.database = Some(database);
    }

    let params = chars.as_str();
    if !params.is_empty() {
        dsn.params = get_params(chars.as_str())?;
    }

    Ok(dsn)
}

/// Example:
///
///```
///use dsn::parse;
///
///let dsn = parse(r#"mysql://user:o%3Ao@tcp(localhost:3306)/database"#).unwrap();
///assert_eq!(dsn.driver, "mysql");
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
///let dsn = parse(r#"mysql://user:o%3Ao@tcp(localhost:3306)/database"#).unwrap();
///assert_eq!(dsn.username.unwrap(), "user");
///assert_eq!(dsn.password.unwrap(), "o:o");
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
        for c in chars {
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
///let dsn = parse(r#"mysql://user:o%3Ao@tcp(localhost:3306)/database"#).unwrap();
///assert_eq!(dsn.protocol, "tcp");
///```
fn get_protocol(chars: &mut Chars) -> Result<String, ParseError> {
    let mut protocol = String::new();
    for c in chars {
        match c {
            '(' => {
                if protocol.is_empty() {
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
///let dsn = parse(r#"mysql://user:o%3Ao@tcp(localhost:3306)/database"#).unwrap();
///assert_eq!(dsn.address, "localhost:3306");
///```
fn get_address(chars: &mut Chars) -> Result<String, ParseError> {
    let mut address = String::new();
    for c in chars {
        match c {
            ')' => {
                if address.is_empty() {
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
///let dsn = parse(r#"mysql://user:o%3Ao@tcp(localhost:3306)/database"#).unwrap();
///assert_eq!(dsn.host.unwrap(), "localhost");
///assert_eq!(dsn.port.unwrap(), 3306);
///```
fn get_host_port(address: &str) -> Result<(String, String), ParseError> {
    let mut host = String::new();
    let mut chars = address.chars();

    // host
    while let Some(c) = chars.next() {
        match c {
            ':' => {
                if host.is_empty() {
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
///let dsn = parse(r#"mysql://user:o%3Ao@tcp(localhost:3306)/database"#).unwrap();
///assert_eq!(dsn.database.unwrap(), "database");
///```
fn get_database(chars: &mut Chars) -> Result<String, ParseError> {
    let mut database = String::new();
    for c in chars {
        match c {
            '/' => {
                if database.is_empty() {
                    continue;
                }
            }
            '?' => break,
            _ => database.push(c),
        }
    }
    Ok(database)
}

/// Example:
///
///```
///use dsn::parse;
///
///let dsn = parse(r#"mysql://user:o%3Ao@tcp(localhost:3306)/database?param1=value1&param2=value2"#).unwrap();
///assert!(!dsn.params.is_empty());
///assert_eq!(dsn.params.get("param1"), Some(&String::from("value1")));
///assert_eq!(dsn.params.get("param2").unwrap(), "value2");
///assert_eq!(dsn.params.get("param3"), None);
///```
fn get_params(params_string: &str) -> Result<BTreeMap<String, String>, ParseError> {
    let params: BTreeMap<String, String> = params_string
        .split('&')
        .map(|kv| kv.split('=').collect::<Vec<&str>>())
        .map(|vec| {
            if vec.len() != 2 {
                return Err(ParseError::InvalidParams);
            }
            Ok((vec[0].to_string(), vec[1].to_string()))
        })
        .collect::<Result<_, _>>()?;
    Ok(params)
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn test_parse_password() {
        let dsn = parse(r#"mysql://user:pas':"'sword44444@host:port/database"#).unwrap();
        assert_eq!(dsn.password.unwrap(), r#"pas':"'sword44444"#);
    }
}
