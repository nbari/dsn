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
//!postgresql://user:pass@tcp(localhost:5432)/dbname
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

use core::str::Utf8Error;
use percent_encoding::percent_decode;
use std::{collections::BTreeMap, error::Error, fmt, str::Chars};

/// Errors that can occur during DSN parsing
#[derive(Debug)]
pub enum ParseError {
    /// Driver name is invalid or missing
    InvalidDriver,
    /// Query parameters are malformed
    InvalidParams,
    /// File path is not absolute
    InvalidPath,
    /// Port number is invalid or out of range
    InvalidPort,
    /// Protocol is invalid or missing
    InvalidProtocol,
    /// Unix socket path is invalid
    InvalidSocket,
    /// Address is missing after protocol
    MissingAddress,
    /// Host is missing in address
    MissingHost,
    /// Protocol is missing
    MissingProtocol,
    /// Unix socket path is missing
    MissingSocket,
    /// UTF-8 decoding error
    Utf8Error(Utf8Error),
}

impl From<Utf8Error> for ParseError {
    fn from(err: Utf8Error) -> Self {
        Self::Utf8Error(err)
    }
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
            Self::Utf8Error(ref err) => write!(f, "UTF-8 error: {err}"),
        }
    }
}

impl Error for ParseError {}

impl fmt::Display for DSN {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};

        write!(f, "{}://", self.driver)?;

        // Add credentials
        if let Some(ref username) = self.username {
            let encoded_user = utf8_percent_encode(username, NON_ALPHANUMERIC);
            write!(f, "{encoded_user}")?;

            if let Some(ref password) = self.password {
                let encoded_pass = utf8_percent_encode(password, NON_ALPHANUMERIC);
                write!(f, ":{encoded_pass}")?;
            }
            write!(f, "@")?;
        }

        // Add protocol and address
        write!(f, "{}({})", self.protocol, self.address)?;

        // Add database
        if let Some(ref database) = self.database {
            write!(f, "/{database}")?;
        }

        // Add parameters
        if !self.params.is_empty() {
            write!(f, "?")?;
            let params: Vec<String> = self
                .params
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect();
            write!(f, "{}", params.join("&"))?;
        }

        Ok(())
    }
}

/// Parsed Data Source Name (DSN) structure
///
/// DSN format: `driver://username:password@protocol(address)/dbname?param=value`
///
/// # Examples
///
/// ```
/// use dsn::parse;
///
/// let dsn = parse("mysql://user:pass@tcp(localhost:3306)/mydb").unwrap();
/// assert_eq!(dsn.driver, "mysql");
/// assert_eq!(dsn.host.unwrap(), "localhost");
/// assert_eq!(dsn.port.unwrap(), 3306);
/// ```
#[derive(Debug, Default)]
pub struct DSN {
    /// Database driver name (e.g., "mysql", "postgres", "sqlite")
    pub driver: String,
    /// Optional username for authentication
    pub username: Option<String>,
    /// Optional password for authentication (percent-decoded)
    pub password: Option<String>,
    /// Connection protocol (e.g., "tcp", "unix", "file")
    pub protocol: String,
    /// Full address string (host:port, socket path, or file path)
    pub address: String,
    /// Hostname (only for TCP/UDP protocols)
    pub host: Option<String>,
    /// Port number (only for TCP/UDP protocols)
    pub port: Option<u16>,
    /// Database name
    pub database: Option<String>,
    /// Unix socket path (only for unix protocol)
    pub socket: Option<String>,
    /// Query string parameters as key-value pairs
    pub params: BTreeMap<String, String>,
}

/// Parse a DSN string into a structured `DSN` object
///
/// This function parses a Data Source Name (DSN) string and extracts all components
/// including driver, credentials, protocol, address, database name, and parameters.
///
/// # Arguments
///
/// * `input` - A DSN string in the format:
///   `driver://username:password@protocol(address)/database?param=value`
///
/// # Returns
///
/// Returns a `Result` containing the parsed `DSN` struct on success, or a
/// `ParseError` if the DSN string is malformed.
///
/// # Errors
///
/// Returns `ParseError` in the following cases:
/// - `InvalidDriver` - Missing or invalid driver name
/// - `InvalidProtocol` - Missing or invalid protocol
/// - `InvalidSocket` - Unix socket path doesn't start with `/`
/// - `InvalidPath` - File path is not absolute
/// - `InvalidPort` - Port number is invalid or out of range (0-65535)
/// - `MissingAddress` - Address is missing after protocol
/// - `MissingHost` - Host is missing in TCP/UDP address
/// - `InvalidParams` - Query parameters are malformed
/// - `Utf8Error` - Percent-encoded credentials contain invalid UTF-8
///
/// # Examples
///
/// Basic TCP connection:
/// ```
/// use dsn::parse;
///
/// let dsn = parse(r#"mysql://user:o%3Ao@tcp(localhost:3306)/database?charset=utf8"#).unwrap();
/// assert_eq!(dsn.driver, "mysql");
/// assert_eq!(dsn.username.unwrap(), "user");
/// assert_eq!(dsn.password.unwrap(), "o:o");
/// assert_eq!(dsn.protocol, "tcp");
/// assert_eq!(dsn.address, "localhost:3306");
/// assert_eq!(dsn.host.unwrap(), "localhost");
/// assert_eq!(dsn.port.unwrap(), 3306);
/// assert_eq!(dsn.database.unwrap(), "database");
/// assert_eq!(dsn.socket, None);
/// assert!(!dsn.params.is_empty());
/// assert_eq!(dsn.params.get("charset").unwrap(), "utf8");
/// ```
///
/// Unix socket connection:
/// ```
/// use dsn::parse;
///
/// let dsn = parse(r"mysql://user@unix(/var/run/mysql.sock)/mydb").unwrap();
/// assert_eq!(dsn.protocol, "unix");
/// assert_eq!(dsn.socket.unwrap(), "/var/run/mysql.sock");
/// ```
pub fn parse(input: &str) -> Result<DSN, ParseError> {
    // create an empty DSN
    let mut dsn = DSN::default();

    // create an iterator for input
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

    match dsn.protocol.as_str() {
        "unix" => {
            if !dsn.address.starts_with('/') {
                return Err(ParseError::InvalidSocket);
            }
            dsn.socket = Some(dsn.address.clone());
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
                dsn.port = Some(port.parse::<u16>().map_err(|_| ParseError::InvalidPort)?);
            }
        }
    }

    // /<database>?
    let database = get_database(chars);
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
    for c in chars.by_ref() {
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

    username = percent_decode(username.as_bytes()).decode_utf8()?.into();

    // password
    if has_password {
        for c in chars {
            match c {
                '@' => break,
                _ => password.push(c),
            }
        }
        password = percent_decode(password.as_bytes()).decode_utf8()?.into();
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
    for c in chars.by_ref() {
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
fn get_database(chars: &mut Chars) -> String {
    let mut database = String::new();
    for c in chars {
        match c {
            '/' if database.is_empty() => {}
            '?' => break,
            _ => database.push(c),
        }
    }
    database
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
    params_string
        .split('&')
        .map(|kv| {
            let parts: Vec<&str> = kv.split('=').collect();
            if parts.len() != 2 {
                return Err(ParseError::InvalidParams);
            }
            Ok((parts[0].to_string(), parts[1].to_string()))
        })
        .collect()
}

impl DSN {
    /// Create a new DSN builder
    ///
    /// # Examples
    ///
    /// ```
    /// use dsn::DSN;
    ///
    /// let dsn = DSN::builder()
    ///     .driver("mysql")
    ///     .username("root")
    ///     .password("secret")
    ///     .host("localhost")
    ///     .port(3306)
    ///     .database("mydb")
    ///     .build();
    ///
    /// assert_eq!(dsn.to_string(), "mysql://root:secret@tcp(localhost:3306)/mydb");
    /// ```
    #[must_use]
    pub fn builder() -> DSNBuilder {
        DSNBuilder::default()
    }
}

/// Builder for constructing DSN strings
///
/// # Examples
///
/// ```
/// use dsn::DSN;
///
/// // MySQL with TCP
/// let mysql = DSN::builder()
///     .driver("mysql")
///     .username("root")
///     .password("secret")
///     .host("localhost")
///     .port(3306)
///     .database("mydb")
///     .param("charset", "utf8mb4")
///     .build();
///
/// // PostgreSQL
/// let postgres = DSN::builder()
///     .driver("postgres")
///     .username("postgres")
///     .password("pass")
///     .host("db.example.com")
///     .port(5432)
///     .database("production")
///     .param("sslmode", "require")
///     .build();
///
/// // Redis
/// let redis = DSN::builder()
///     .driver("redis")
///     .host("localhost")
///     .port(6379)
///     .database("0")
///     .build();
///
/// // MySQL with Unix socket
/// let mysql_sock = DSN::builder()
///     .driver("mysql")
///     .username("app")
///     .socket("/var/run/mysqld/mysqld.sock")
///     .database("appdb")
///     .build();
/// ```
#[derive(Debug, Default)]
pub struct DSNBuilder {
    driver: String,
    username: Option<String>,
    password: Option<String>,
    protocol: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    socket: Option<String>,
    database: Option<String>,
    params: BTreeMap<String, String>,
}

impl DSNBuilder {
    /// Set the database driver (e.g., "mysql", "postgres", "redis")
    #[must_use]
    pub fn driver(mut self, driver: impl Into<String>) -> Self {
        self.driver = driver.into();
        self
    }

    /// Set the username for authentication
    #[must_use]
    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    /// Set the password for authentication
    #[must_use]
    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    /// Set the host for TCP connection
    #[must_use]
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self.protocol = Some("tcp".to_string());
        self
    }

    /// Set the port for TCP connection
    #[must_use]
    pub const fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    /// Set a Unix socket path
    #[must_use]
    pub fn socket(mut self, socket: impl Into<String>) -> Self {
        self.socket = Some(socket.into());
        self.protocol = Some("unix".to_string());
        self
    }

    /// Set the database name
    #[must_use]
    pub fn database(mut self, database: impl Into<String>) -> Self {
        self.database = Some(database.into());
        self
    }

    /// Add a query parameter
    #[must_use]
    pub fn param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }

    /// Build the DSN
    #[must_use]
    pub fn build(self) -> DSN {
        let protocol = self.protocol.unwrap_or_else(|| "tcp".to_string());

        let (address, host, socket) = if let Some(socket_path) = self.socket {
            // Unix socket
            (socket_path.clone(), None, Some(socket_path))
        } else {
            // TCP/UDP
            let host_name = self.host.clone().unwrap_or_else(|| "localhost".to_string());
            let addr = self
                .port
                .map_or_else(|| host_name.clone(), |port| format!("{host_name}:{port}"));
            (addr, Some(host_name), None)
        };

        DSN {
            driver: self.driver,
            username: self.username,
            password: self.password,
            protocol,
            address,
            host,
            port: self.port,
            database: self.database,
            socket,
            params: self.params,
        }
    }
}

impl DSNBuilder {
    /// Create a MySQL/MariaDB DSN builder with common defaults
    ///
    /// # Examples
    ///
    /// ```
    /// use dsn::DSNBuilder;
    ///
    /// let dsn = DSNBuilder::mysql()
    ///     .username("root")
    ///     .password("secret")
    ///     .host("localhost")
    ///     .database("mydb")
    ///     .build();
    ///
    /// assert_eq!(dsn.driver, "mysql");
    /// assert_eq!(dsn.port, Some(3306));
    /// ```
    #[must_use]
    pub fn mysql() -> Self {
        Self {
            driver: "mysql".to_string(),
            protocol: Some("tcp".to_string()),
            port: Some(3306),
            ..Default::default()
        }
    }

    /// Create a `PostgreSQL` DSN builder with common defaults
    ///
    /// # Examples
    ///
    /// ```
    /// use dsn::DSNBuilder;
    ///
    /// let dsn = DSNBuilder::postgres()
    ///     .username("postgres")
    ///     .password("pass")
    ///     .host("localhost")
    ///     .database("mydb")
    ///     .build();
    ///
    /// assert_eq!(dsn.driver, "postgres");
    /// assert_eq!(dsn.port, Some(5432));
    /// ```
    #[must_use]
    pub fn postgres() -> Self {
        Self {
            driver: "postgres".to_string(),
            protocol: Some("tcp".to_string()),
            port: Some(5432),
            ..Default::default()
        }
    }

    /// Create a Redis DSN builder with common defaults
    ///
    /// # Examples
    ///
    /// ```
    /// use dsn::DSNBuilder;
    ///
    /// let dsn = DSNBuilder::redis()
    ///     .host("localhost")
    ///     .password("secret")
    ///     .database("0")
    ///     .build();
    ///
    /// assert_eq!(dsn.driver, "redis");
    /// assert_eq!(dsn.port, Some(6379));
    /// ```
    #[must_use]
    pub fn redis() -> Self {
        Self {
            driver: "redis".to_string(),
            protocol: Some("tcp".to_string()),
            port: Some(6379),
            ..Default::default()
        }
    }

    /// Create a `MariaDB` DSN builder (alias for `MySQL`)
    ///
    /// # Examples
    ///
    /// ```
    /// use dsn::DSNBuilder;
    ///
    /// let dsn = DSNBuilder::mariadb()
    ///     .username("root")
    ///     .host("localhost")
    ///     .database("mydb")
    ///     .build();
    ///
    /// assert_eq!(dsn.driver, "mariadb");
    /// assert_eq!(dsn.port, Some(3306));
    /// ```
    #[must_use]
    pub fn mariadb() -> Self {
        Self {
            driver: "mariadb".to_string(),
            protocol: Some("tcp".to_string()),
            port: Some(3306),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DSN, DSNBuilder, ParseError, parse};

    #[test]
    fn test_parse_password() {
        let dsn = parse(r#"mysql://user:pas':"'sword44444@host:port/database"#).unwrap();
        assert_eq!(dsn.password.unwrap(), r#"pas':"'sword44444"#);
    }

    #[test]
    fn test_parse_driver() {
        let dsn = parse(r"mysql://user:pass@host:port/database").unwrap();
        assert_eq!(dsn.driver, "mysql");
    }

    #[test]
    fn test_parse_driver_postgres() {
        let dsn = parse(r"postgres://user:pass@host:port/database").unwrap();
        assert_eq!(dsn.driver, "postgres");
    }

    #[test]
    fn test_parse_username() {
        let dsn = parse(r"mysql://user:pass@host:port/database").unwrap();
        assert_eq!(dsn.username.unwrap(), "user");
    }

    #[test]
    fn test_parse_protocol() {
        let dsn = parse(r"mysql://user:pass@tcp(host:3306)/database").unwrap();
        assert_eq!(dsn.protocol, "tcp");
    }

    #[test]
    fn test_parse_address() {
        let dsn = parse(r"mysql://user:pass@tcp(host:3306)/database").unwrap();
        assert_eq!(dsn.address, "host:3306");
    }

    #[test]
    fn test_parse_host() {
        let dsn = parse(r"mysql://user:pass@tcp(host:3306)/database").unwrap();
        assert_eq!(dsn.host.unwrap(), "host");
    }

    #[test]
    fn test_parse_port() {
        let dsn = parse(r"mysql://user:pass@tcp(host:3306)/database").unwrap();
        assert_eq!(dsn.port.unwrap(), 3306);
    }

    #[test]
    fn test_builder_mysql() {
        let dsn = DSNBuilder::mysql()
            .username("root")
            .password("secret")
            .host("localhost")
            .database("mydb")
            .param("charset", "utf8mb4")
            .build();

        assert_eq!(dsn.driver, "mysql");
        assert_eq!(dsn.username.as_deref(), Some("root"));
        assert_eq!(dsn.password.as_deref(), Some("secret"));
        assert_eq!(dsn.host.as_deref(), Some("localhost"));
        assert_eq!(dsn.port, Some(3306));
        assert_eq!(dsn.database.as_deref(), Some("mydb"));
        assert_eq!(dsn.params.get("charset"), Some(&"utf8mb4".to_string()));
    }

    #[test]
    fn test_builder_postgres() {
        let dsn = DSNBuilder::postgres()
            .username("postgres")
            .password("pass")
            .host("db.example.com")
            .database("production")
            .param("sslmode", "require")
            .build();

        assert_eq!(dsn.driver, "postgres");
        assert_eq!(dsn.port, Some(5432));
        assert_eq!(dsn.params.get("sslmode"), Some(&"require".to_string()));
    }

    #[test]
    fn test_builder_redis() {
        let dsn = DSNBuilder::redis()
            .host("localhost")
            .password("secret")
            .database("0")
            .build();

        assert_eq!(dsn.driver, "redis");
        assert_eq!(dsn.port, Some(6379));
        assert_eq!(dsn.database.as_deref(), Some("0"));
    }

    #[test]
    fn test_builder_unix_socket() {
        let dsn = DSNBuilder::mysql()
            .username("app")
            .socket("/var/run/mysqld/mysqld.sock")
            .database("appdb")
            .build();

        assert_eq!(dsn.protocol, "unix");
        assert_eq!(dsn.socket.as_deref(), Some("/var/run/mysqld/mysqld.sock"));
        assert_eq!(dsn.address, "/var/run/mysqld/mysqld.sock");
    }

    #[test]
    fn test_to_string_basic() {
        let dsn = DSNBuilder::mysql()
            .username("root")
            .password("secret")
            .host("localhost")
            .database("mydb")
            .build();

        let dsn_string = dsn.to_string();
        assert!(dsn_string.contains("mysql://"));
        assert!(dsn_string.contains("root"));
        assert!(dsn_string.contains("secret"));
        assert!(dsn_string.contains("localhost:3306"));
        assert!(dsn_string.contains("/mydb"));
    }

    #[test]
    fn test_to_string_with_params() {
        let dsn = DSNBuilder::postgres()
            .username("user")
            .password("pass")
            .host("localhost")
            .database("db")
            .param("sslmode", "require")
            .param("connect_timeout", "10")
            .build();

        let dsn_string = dsn.to_string();
        assert!(dsn_string.contains('?'));
        assert!(dsn_string.contains("sslmode=require"));
        assert!(dsn_string.contains("connect_timeout=10"));
    }

    #[test]
    fn test_to_string_special_chars() {
        let dsn = DSNBuilder::mysql()
            .username("user@host")
            .password("p@ss:word!")
            .host("localhost")
            .database("mydb")
            .build();

        let dsn_string = dsn.to_string();
        // Should be percent-encoded
        assert!(dsn_string.contains("%40")); // @
        assert!(!dsn_string.contains("user@host"));
    }

    #[test]
    fn test_roundtrip() {
        let original = "mysql://root:secret@tcp(localhost:3306)/mydb?charset=utf8mb4";
        let parsed = parse(original).unwrap();
        let rebuilt = parsed.to_string();

        // Parse the rebuilt string to verify it's valid
        let reparsed = parse(&rebuilt).unwrap();
        assert_eq!(parsed.driver, reparsed.driver);
        assert_eq!(parsed.username, reparsed.username);
        assert_eq!(parsed.host, reparsed.host);
        assert_eq!(parsed.port, reparsed.port);
        assert_eq!(parsed.database, reparsed.database);
    }

    #[test]
    fn test_builder_mariadb() {
        let dsn = DSNBuilder::mariadb()
            .username("root")
            .host("localhost")
            .database("mydb")
            .build();

        assert_eq!(dsn.driver, "mariadb");
        assert_eq!(dsn.port, Some(3306));
    }

    #[test]
    fn test_error_display() {
        // Test all error display messages
        assert_eq!(format!("{}", ParseError::InvalidDriver), "invalid driver");
        assert_eq!(format!("{}", ParseError::InvalidParams), "invalid params");
        assert_eq!(
            format!("{}", ParseError::InvalidPath),
            "invalid absolute path"
        );
        assert_eq!(
            format!("{}", ParseError::InvalidPort),
            "invalid port number"
        );
        assert_eq!(
            format!("{}", ParseError::InvalidProtocol),
            "invalid protocol"
        );
        assert_eq!(format!("{}", ParseError::InvalidSocket), "invalid socket");
        assert_eq!(format!("{}", ParseError::MissingAddress), "missing address");
        assert_eq!(format!("{}", ParseError::MissingHost), "missing host");
        assert_eq!(
            format!("{}", ParseError::MissingProtocol),
            "missing protocol"
        );
        assert_eq!(
            format!("{}", ParseError::MissingSocket),
            "missing unix domain socket"
        );
    }

    #[test]
    #[allow(invalid_from_utf8)]
    fn test_utf8_error_from() {
        // Test Utf8Error conversion
        let bad_bytes: &[u8] = &[0xFF, 0xFF];
        let utf8_err = std::str::from_utf8(bad_bytes).unwrap_err();
        let parse_err = ParseError::from(utf8_err);
        match parse_err {
            ParseError::Utf8Error(_) => {
                assert!(format!("{parse_err}").contains("UTF-8 error"));
            }
            _ => panic!("Expected Utf8Error variant"),
        }
    }

    #[test]
    fn test_to_string_no_credentials() {
        // Test DSN without username/password
        let dsn = DSNBuilder::mysql().host("localhost").database("db").build();

        let dsn_string = dsn.to_string();
        assert!(dsn_string.contains("mysql://"));
        assert!(!dsn_string.contains('@')); // No @ if no credentials
        assert!(dsn_string.contains("tcp(localhost:3306)"));
    }

    #[test]
    fn test_to_string_no_database() {
        // Test DSN without database
        let dsn = DSNBuilder::mysql()
            .username("root")
            .password("pass")
            .host("localhost")
            .build();

        let dsn_string = dsn.to_string();
        assert!(dsn_string.contains("mysql://"));
        assert!(dsn_string.ends_with("tcp(localhost:3306)")); // No trailing /
    }

    #[test]
    fn test_to_string_username_only() {
        // Test DSN with username but no password
        let dsn = DSNBuilder::mysql()
            .username("root")
            .host("localhost")
            .database("db")
            .build();

        let dsn_string = dsn.to_string();
        assert!(dsn_string.contains("mysql://root@"));
        assert!(!dsn_string.contains(":@")); // No colon before @
    }

    #[test]
    fn test_builder_default() {
        // Test default builder
        let dsn = DSNBuilder::default()
            .driver("custom")
            .host("localhost")
            .port(9999)
            .build();

        assert_eq!(dsn.driver, "custom");
        assert_eq!(dsn.port, Some(9999));
    }

    #[test]
    fn test_builder_const_port() {
        // Test const port function
        let dsn = DSNBuilder::mysql().port(3307).host("localhost").build();

        assert_eq!(dsn.port, Some(3307));
    }

    #[test]
    fn test_parse_errors() {
        // Test various parse errors
        assert!(parse("mysql://user@tcp(host:99999)/db").is_err()); // Port out of range
        assert!(parse("mysql://user@unix(relative/path)/db").is_err()); // Unix socket must be absolute
        assert!(parse("mysql://user@file(relative/path)/db").is_err()); // File path must be absolute
        assert!(parse("mysql://user@tcp()/db").is_err()); // Missing address
        assert!(parse("mysql://user@tcp(:3306)/db").is_err()); // Missing host
        assert!(parse("mysql://user@tcp(host:port)/db").is_err()); // Invalid port (not a number)
    }

    #[test]
    fn test_parse_edge_cases() {
        // These should parse but have empty driver
        let dsn = parse("://user@tcp(host)/db").unwrap();
        assert_eq!(dsn.driver, "");

        // Test protocol variations work
        let dsn = parse("mysql://user@udp(host:9999)/db").unwrap();
        assert_eq!(dsn.protocol, "udp");
    }

    #[test]
    fn test_parse_missing_protocol() {
        // Test missing protocol before (
        assert!(parse("mysql://user@(host)/db").is_err());
    }

    #[test]
    fn test_dsn_builder_method() {
        // Test DSN::builder() method
        let dsn = DSN::builder().driver("mysql").host("localhost").build();

        assert_eq!(dsn.driver, "mysql");
    }
}
