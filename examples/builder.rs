//! Examples of building DSN strings for various databases

use dsn::DSNBuilder;

fn main() {
    println!("=== DSN Builder Examples ===\n");

    // MySQL / MariaDB
    println!("1. MySQL with TCP:");
    let mysql = DSNBuilder::mysql()
        .username("root")
        .password("my-secret-pw")
        .host("db.example.com")
        .database("myapp")
        .param("charset", "utf8mb4")
        .param("parseTime", "true")
        .build();
    println!("   {mysql}\n");

    // MySQL with Unix socket
    println!("2. MySQL with Unix socket:");
    let mysql_sock = DSNBuilder::mysql()
        .username("webapp")
        .password("secret")
        .socket("/var/run/mysqld/mysqld.sock")
        .database("webapp_db")
        .build();
    println!("   {mysql_sock}\n");

    // PostgreSQL
    println!("3. PostgreSQL with SSL:");
    let postgres = DSNBuilder::postgres()
        .username("postgres")
        .password("admin123")
        .host("postgres.example.com")
        .port(5432)
        .database("production")
        .param("sslmode", "require")
        .param("connect_timeout", "10")
        .build();
    println!("   {postgres}\n");

    // PostgreSQL with custom port
    println!("4. PostgreSQL with custom port:");
    let postgres_custom = DSNBuilder::postgres()
        .username("user")
        .password("pass")
        .host("localhost")
        .port(5433)
        .database("testdb")
        .build();
    println!("   {postgres_custom}\n");

    // PostgreSQL with SSL disabled (development)
    println!("5. PostgreSQL with SSL disabled:");
    let postgres_no_ssl = DSNBuilder::postgres()
        .username("dev")
        .password("dev123")
        .host("localhost")
        .database("dev_db")
        .param("sslmode", "disable")
        .build();
    println!("   {postgres_no_ssl}\n");

    // Redis
    println!("6. Redis:");
    let redis = DSNBuilder::redis()
        .host("cache.example.com")
        .password("redis-secret")
        .database("0")
        .build();
    println!("   {redis}\n");

    // Redis with authentication
    println!("7. Redis with custom port:");
    let redis_custom = DSNBuilder::redis()
        .host("localhost")
        .port(6380)
        .database("1")
        .build();
    println!("   {redis_custom}\n");

    // MariaDB
    println!("8. MariaDB:");
    let mariadb = DSNBuilder::mariadb()
        .username("admin")
        .password("secure-password")
        .host("mariadb.local")
        .database("app_data")
        .param("timeout", "30s")
        .build();
    println!("   {mariadb}\n");

    // Generic builder
    println!("9. Custom driver (MongoDB):");
    let mongo = DSNBuilder::default()
        .driver("mongodb")
        .username("admin")
        .password("admin")
        .host("mongo1.example.com")
        .port(27017)
        .database("admin")
        .param("replicaSet", "rs0")
        .param("authSource", "admin")
        .build();
    println!("   {mongo}\n");

    // Special characters in password
    println!("10. Password with special characters:");
    let special = DSNBuilder::mysql()
        .username("user@domain")
        .password("p@ss:w0rd!#$")
        .host("localhost")
        .database("mydb")
        .build();
    println!("   {special}\n");
    println!("    (Note: special characters are automatically percent-encoded)");
}
