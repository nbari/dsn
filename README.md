# DSN

DSN (Data Source Name) parser

DSN format:

    <driver>://<username>:<password>@<host>:<port>/<database>

When using a Unix domain socket:

    <driver>://<username>:<password>@unix:<socket>/<database>

Extra params are optional, for example:

    <driver>://<username>:password>@unix:<socket>/<database>[?param1=value1&...&paramN=valueN]

## Defaults

Based on the `driver` the defaults are set.

for example:

    mysql://user:password@localhost/

If port is omitted, and the driver is `mysql` port `3306` will be used.

For redis:

    redis://localhost

port will be `6379` and database number `0`
