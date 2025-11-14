//! Example showing `PostgreSQL` DSN with different SSL modes

use dsn::{DSNBuilder, parse};

fn main() {
    println!("=== PostgreSQL SSL Mode Examples ===\n");

    // Building DSNs with different SSL modes
    println!("1. Building PostgreSQL DSNs:\n");

    let ssl_require = DSNBuilder::postgres()
        .username("app")
        .password("secret")
        .host("prod.postgres.com")
        .database("app_db")
        .param("sslmode", "require")
        .build();
    println!("   Production (SSL required):");
    println!("   {ssl_require}\n");

    let ssl_prefer = DSNBuilder::postgres()
        .username("app")
        .password("secret")
        .host("staging.postgres.com")
        .database("app_db")
        .param("sslmode", "prefer")
        .build();
    println!("   Staging (SSL preferred):");
    println!("   {ssl_prefer}\n");

    let ssl_disable = DSNBuilder::postgres()
        .username("dev")
        .password("dev123")
        .host("localhost")
        .database("dev_db")
        .param("sslmode", "disable")
        .build();
    println!("   Development (SSL disabled):");
    println!("   {ssl_disable}\n");

    // Parsing and checking SSL mode
    println!("2. Parsing PostgreSQL DSN and checking SSL mode:\n");

    let examples = [
        "postgres://user:pass@tcp(localhost:5432)/mydb?sslmode=disable",
        "postgres://user:pass@tcp(prod.db.com:5432)/mydb?sslmode=require",
        "postgres://user:pass@tcp(stage.db.com:5432)/mydb?sslmode=prefer&connect_timeout=10",
    ];

    for dsn_str in examples {
        match parse(dsn_str) {
            Ok(dsn) => {
                println!("   DSN: {dsn_str}");
                println!("   Host: {}", dsn.host.as_ref().unwrap());
                println!("   Database: {}", dsn.database.as_ref().unwrap());

                if let Some(sslmode) = dsn.params.get("sslmode") {
                    println!("   SSL Mode: {sslmode}");

                    match sslmode.as_str() {
                        "disable" => {
                            println!(
                                "   [WARNING] SSL is disabled - not recommended for production!"
                            );
                        }
                        "require" => println!("   [OK] SSL is required - secure connection"),
                        "prefer" => println!("   [INFO] SSL is preferred - will use if available"),
                        "verify-ca" => println!("   [SECURE] SSL with CA verification"),
                        "verify-full" => println!("   [SECURE] SSL with full verification"),
                        _ => println!("   [UNKNOWN] Unknown SSL mode: {sslmode}"),
                    }
                } else {
                    println!("   [INFO] No SSL mode specified (will use PostgreSQL default)");
                }

                // Check for other connection parameters
                if let Some(timeout) = dsn.params.get("connect_timeout") {
                    println!("   Connection timeout: {timeout}s");
                }

                println!();
            }
            Err(e) => {
                eprintln!("   [ERROR] Failed to parse: {e}");
            }
        }
    }

    // Practical usage example
    println!("3. Practical usage - connection string selection:\n");

    let environment = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

    let dsn = match environment.as_str() {
        "production" => DSNBuilder::postgres()
            .username("prod_user")
            .password("prod_pass")
            .host("prod.postgres.com")
            .database("prod_db")
            .param("sslmode", "require")
            .param("connect_timeout", "30")
            .build(),

        "staging" => DSNBuilder::postgres()
            .username("stage_user")
            .password("stage_pass")
            .host("staging.postgres.com")
            .database("stage_db")
            .param("sslmode", "prefer")
            .param("connect_timeout", "10")
            .build(),

        _ => DSNBuilder::postgres()
            .username("dev")
            .password("dev")
            .host("localhost")
            .database("dev_db")
            .param("sslmode", "disable")
            .build(),
    };

    println!("   Environment: {environment}");
    println!("   Connection string: {dsn}");
}
