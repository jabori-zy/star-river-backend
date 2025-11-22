# Running Migrator CLI

- Generate a new migration file
    ```sh
    cargo run -- generate MIGRATION_NAME
    ```
- Apply all pending migrations
    ```sh
    cargo run
    ```
    ```sh
    cargo run -- up
    ```
- Apply first 10 pending migrations
    ```sh
    cargo run -- up -n 10
    ```
- Rollback last applied migrations
    ```sh
    cargo run -- down
    ```
- Rollback last 10 applied migrations
    ```sh
    cargo run -- down -n 10
    ```
- Drop all tables from the database, then reapply all migrations
    ```sh
    cargo run -- fresh
    ```
- Rollback all applied migrations, then reapply all migrations
    ```sh
    cargo run -- refresh
    ```
- Rollback all applied migrations
    ```sh
    cargo run -- reset
    ```
- Check the status of all migrations
    ```sh
    cargo run -- status
    ```
use chrono::Utc;

let current_time = Utc::now();
ColumnDef::new(SystemConfig::CreatedTime)
    .timestamp()
    .not_null()
    .default(SimpleExpr::Value(current_time.into())),


use sea_orm_migration::prelude::*;

ColumnDef::new(SystemConfig::CreatedTime)
    .timestamp()
    .not_null()
    .default(Expr::current_timestamp()),