# DSN (Data Source Name) parser

[![crates.io](https://img.shields.io/crates/v/dsn.svg)](https://crates.io/crates/dsn)
[![Build Status](https://travis-ci.org/nbari/dsn.svg?branch=master)](https://travis-ci.org/nbari/dsn)


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
