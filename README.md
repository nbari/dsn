# DSN (Data Source Name) parser

[![crates.io](https://img.shields.io/crates/v/dsn.svg)](https://crates.io/crates/dsn)
[![Test & Build](https://github.com/nbari/dsn/actions/workflows/build.yml/badge.svg)](https://github.com/nbari/dsn/actions/workflows/build.yml)
[![docs](https://docs.rs/dsn/badge.svg)](https://docs.rs/dsn)


DSN format:

    <driver>://<username>:<password>@<protocol>(<address>)/<database>?param1=value1&...&paramN=valueN

A DSN in its fullest form:

    driver://username:password@protocol(address)/dbname?param=value

The address changes depending on the protocol

For `TCP/UDP` address have the form `host:port`, example:

    pgsql://user:pass@tcp(localhost:5555)/dbname

For protocol `unix` (Unix domain sockets) the address is the absolute path to the socket, for example:

    mysql://user@unix(/path/to/socket)/database

For protocol `file` (sqlite) use the absolute path as the address, example:

    sqlite://@file(/full/unix/path/to/file.db)

# percent-encode

Percent-encode username and password with characters like `@`, for example if password is:

    !A4T@hh'cUj7LXXvk"

From the command line you can encode it with:

    echo -n "\!A4T@hh'cUj7LXXvk\"" | jq -s -R -r @uri

or

    echo -n "\!A4T@hh'cUj7LXXvk\"" | xxd -p |sed 's/../%&/g'

Then you can build the dsn:


    mysql://root:!A4T%40hh'cUj7LXXvk%22@tcp(10.0.0.1:3306)/test

or

    mysql://root:%21%41%34%54%40%68%68%27%63%55%6a%37%4c%58%58%76%6b%22@tcp(10.0.0.1:3306)/test



# Example using the mysql create [![crates.io](https://img.shields.io/crates/v/mysql.svg)](https://crates.io/crates/mysql)

DSN:

    mysql://user:password@tcp(db.example.com)/mydb?tls=skip-verify

Code:

    // if using clap asking for the DSN as an argument
    let dsn = matches.value_of("DSN").unwrap();
    let dsn = dsn::parse(dsn).unwrap_or_else(|e| {
        eprintln!("{}", e);
        process::exit(1);
    });

    let mut opts = mysql::OptsBuilder::new();
    opts.user(dsn.username);
    opts.pass(dsn.password);
    opts.ip_or_hostname(dsn.host);
    if let Some(port) = dsn.port {
        opts.tcp_port(port);
    }
    opts.socket(dsn.socket);
    opts.db_name(dsn.database);

    // mysql ssl options
    let mut ssl_opts = mysql::SslOpts::default();
    if let Some(tls) = dsn.params.get("tls") {
        if *tls == "skip-verify" {
            ssl_opts.set_danger_accept_invalid_certs(true);
        }
    }
    opts.ssl_opts(ssl_opts);

    let pool = mysql::Pool::new_manual(3, 50, opts).unwrap_or_else(|e| {
        eprintln!("Could not connect to MySQL: {}", e);
        process::exit(1);
    });
