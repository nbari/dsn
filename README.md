# DSN

DSN (Data Source Name) parser

DSN format:

    <driver>://<username>:<password>@<host>:<port>/<database>[?param1=value1&...&paramN=valueN]

When using a Unix domain socket:

    <driver>://<username>:<password>@unix:<socket>/<database>[?param1=value1&...&paramN=valueN]


## Defaults

Based on the `driver` the defaults are set.

for example:

    mysql://user:password@localhost/

If port is omitted, andthe driver is `mysql` port `3306` will be used.

For redis:

    redis://localhost

port will be `6379` and database number 0
