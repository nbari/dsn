use dsn::parse;

#[test]
// Connect to database through a socket
fn test_parse_driver1() {
    let dsn = parse(r#"mysql://user@unix(/path/to/socket)/pear"#).unwrap();
    assert_eq!(dsn.driver, "mysql");
    assert_eq!(dsn.username.unwrap(), "user");
    assert_eq!(dsn.password, None);
    assert_eq!(dsn.protocol, "unix");
    assert_eq!(dsn.address, "/path/to/socket");
    assert_eq!(dsn.host, None);
    assert_eq!(dsn.port, None);
    assert_eq!(dsn.database.unwrap(), "pear");
    assert_eq!(dsn.socket.unwrap(), "/path/to/socket");
    assert!(dsn.params.is_empty());
}

#[test]
// Connect to database on a non standard port
fn test_parse_driver2() {
    let dsn = parse(r#"pgsql://user:pass@tcp(localhost:5555)/pear"#).unwrap();
    println!("{:#?}", dsn);
    assert_eq!(dsn.driver, "pgsql");
    assert_eq!(dsn.username.unwrap(), "user");
    assert_eq!(dsn.password.unwrap(), "pass");
    assert_eq!(dsn.protocol, "tcp");
    assert_eq!(dsn.address, "localhost:5555");
    assert_eq!(dsn.host.unwrap(), "localhost");
    assert_eq!(dsn.port.unwrap(), 5555);
    assert_eq!(dsn.database.unwrap(), "pear");
    assert_eq!(dsn.socket, None);
    assert!(dsn.params.is_empty());
}

#[test]
// Connect to database on a non standard port
fn test_parse_driver3() {
    let dsn = parse(r#"sqlite://@file(/full/unix/path/to/file.db)/?mode=0660"#).unwrap();
    println!("{:#?}", dsn);
    assert_eq!(dsn.driver, "sqlite");
    assert_eq!(dsn.username, None);
    assert_eq!(dsn.password, None);
    assert_eq!(dsn.protocol, "file");
    assert_eq!(dsn.address, "/full/unix/path/to/file.db");
    assert_eq!(dsn.host, None);
    assert_eq!(dsn.port, None);
    assert_eq!(dsn.database, None);
    assert_eq!(dsn.socket, None);
    assert!(!dsn.params.is_empty());
    assert_eq!(dsn.params.get("mode").unwrap(), "0660");
}

#[test]
// Missing params
fn test_parse_driver4() {
    assert!(
        parse(r#"mysql://@unix(/tmp/mysql.sock)/?tls=false&cert"#).is_err(),
        "params are wrong"
    );
}

#[test]
// empty params
fn test_parse_driver5() {
    let dsn = parse(r#"mysql://@unix(/tmp/mysql.sock)/?charset=utf8&tls=false&cert="#).unwrap();
    println!("{:#?}", dsn);
    assert_eq!(dsn.driver, "mysql");
    assert_eq!(dsn.username, None);
    assert_eq!(dsn.password, None);
    assert_eq!(dsn.protocol, "unix");
    assert_eq!(dsn.address, "/tmp/mysql.sock");
    assert_eq!(dsn.host, None);
    assert_eq!(dsn.port, None);
    assert_eq!(dsn.database, None);
    assert_eq!(dsn.socket.unwrap(), "/tmp/mysql.sock");
    assert!(!dsn.params.is_empty());
    assert_eq!(dsn.params.get("charset").unwrap(), "utf8");
    assert_eq!(dsn.params.get("tls").unwrap(), "false");
}
