# DSN (Data Source Name) parser

[![crates.io](https://img.shields.io/crates/v/dsn.svg)](https://crates.io/crates/dsn)
[![Build Status](https://travis-ci.org/nbari/dsn.svg?branch=master)](https://travis-ci.org/nbari/dsn)


DSN format:

    <driver>://<username>:<password>@<protocol>(<address>)/<database>?param1=value1&...&paramN=valueN

A DSN in its fullest form:

    driver://username:password@protocol(address)/dbname?param=value

For protocol `TCP/UDP` address have the form `host:port`.

For protocol `unix` (Unix domain sockets) the address is the absolute path to the socket.

Connect to database on a non standard port:

    pgsql://user:pass@tcp(localhost:5555)/dbname

When using a Unix domain socket:

    mysql://user@unix(/path/to/socket)/database
