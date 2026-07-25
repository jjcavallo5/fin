use sea_orm::{
    ConnectionTrait, DatabaseBackend, DatabaseConnection, DbErr, Statement, TransactionTrait,
};

const SCHEMA_VERSION: i64 = 1;

const INITIAL_SCHEMA: &[&str] = &[
    r#"CREATE TABLE fin_schema_migrations (
        version INTEGER PRIMARY KEY,
        applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
    )"#,
    r#"CREATE TABLE encryption (
        salt TEXT PRIMARY KEY NOT NULL
    )"#,
    r#"CREATE TABLE plaid_item (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        institution_name TEXT NOT NULL,
        nonce TEXT NOT NULL,
        encrypted_token TEXT NOT NULL
    )"#,
    r#"CREATE TABLE asset_accounts (
        account_id TEXT PRIMARY KEY NOT NULL,
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        name TEXT NOT NULL,
        account_type TEXT NOT NULL CHECK (account_type IN ('depository', 'investment', 'brokerage', 'other')),
        account_subtype TEXT NOT NULL,
        plaid_item_id INTEGER NOT NULL,
        FOREIGN KEY (plaid_item_id) REFERENCES plaid_item(id) ON DELETE CASCADE
    )"#,
    r#"CREATE TABLE liability_accounts (
        account_id TEXT PRIMARY KEY NOT NULL,
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        name TEXT NOT NULL,
        account_type TEXT NOT NULL CHECK (account_type IN ('credit', 'loan')),
        account_subtype TEXT NOT NULL,
        plaid_item_id INTEGER NOT NULL,
        FOREIGN KEY (plaid_item_id) REFERENCES plaid_item(id) ON DELETE CASCADE
    )"#,
    r#"CREATE TABLE plans (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        plan_type TEXT NOT NULL CHECK (plan_type IN ('recurring')),
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
    )"#,
    r#"CREATE TABLE asset_balance_rules (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        minimum_balance_cents INTEGER NOT NULL CHECK (minimum_balance_cents >= 0),
        plan_id INTEGER NOT NULL,
        asset_account_id TEXT NOT NULL,
        UNIQUE (plan_id, asset_account_id),
        FOREIGN KEY (plan_id) REFERENCES plans(id) ON DELETE CASCADE,
        FOREIGN KEY (asset_account_id) REFERENCES asset_accounts(account_id)
    )"#,
    r#"CREATE TABLE plan_liability_rules (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        rule_type TEXT NOT NULL CHECK (rule_type IN ('target_balance', 'fixed_payment')),
        value_cents INTEGER NOT NULL CHECK (value_cents >= 0),
        requirement TEXT NOT NULL CHECK (requirement IN ('required', 'best_effort')),
        position INTEGER NOT NULL CHECK (position >= 0),
        plan_id INTEGER NOT NULL,
        liability_account_id TEXT NOT NULL,
        payment_asset_account_id TEXT NOT NULL,
        UNIQUE (plan_id, liability_account_id),
        UNIQUE (plan_id, position),
        FOREIGN KEY (plan_id) REFERENCES plans(id) ON DELETE CASCADE,
        FOREIGN KEY (liability_account_id) REFERENCES liability_accounts(account_id),
        FOREIGN KEY (payment_asset_account_id) REFERENCES asset_accounts(account_id)
    )"#,
    r#"CREATE TABLE plan_excess_allocation_rules (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        allocation_type TEXT NOT NULL CHECK (allocation_type IN ('fixed_amount', 'percentage', 'remainder')),
        allocation_value INTEGER,
        position INTEGER NOT NULL CHECK (position >= 0),
        plan_id INTEGER NOT NULL,
        source_asset_account_id TEXT NOT NULL,
        destination_asset_account_id TEXT NOT NULL,
        CHECK (
            (allocation_type = 'fixed_amount' AND allocation_value IS NOT NULL AND allocation_value >= 0)
            OR (allocation_type = 'percentage' AND allocation_value IS NOT NULL AND allocation_value BETWEEN 0 AND 10000)
            OR (allocation_type = 'remainder' AND allocation_value IS NULL)
        ),
        CHECK (source_asset_account_id <> destination_asset_account_id),
        UNIQUE (plan_id, position),
        FOREIGN KEY (plan_id) REFERENCES plans(id) ON DELETE CASCADE,
        FOREIGN KEY (source_asset_account_id) REFERENCES asset_accounts(account_id),
        FOREIGN KEY (destination_asset_account_id) REFERENCES asset_accounts(account_id)
    )"#,
    r#"CREATE UNIQUE INDEX one_remainder_per_plan_source
        ON plan_excess_allocation_rules(plan_id, source_asset_account_id)
        WHERE allocation_type = 'remainder'"#,
];

pub async fn migrate(
    db: &DatabaseConnection,
    database_path: &std::path::Path,
) -> Result<(), DbErr> {
    db.execute_raw(Statement::from_string(
        DatabaseBackend::Sqlite,
        "PRAGMA foreign_keys = ON",
    ))
    .await?;

    let existing_tables = db
        .query_one_raw(Statement::from_string(
            DatabaseBackend::Sqlite,
            "SELECT COUNT(*) AS count FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
        ))
        .await?
        .ok_or_else(|| DbErr::Custom("failed to inspect database schema".to_string()))?
        .try_get::<i64>("", "count")?;

    let has_migrations = db
        .query_one_raw(Statement::from_string(
            DatabaseBackend::Sqlite,
            "SELECT COUNT(*) AS count FROM sqlite_master WHERE type = 'table' AND name = 'fin_schema_migrations'",
        ))
        .await?
        .ok_or_else(|| DbErr::Custom("failed to inspect migration state".to_string()))?
        .try_get::<i64>("", "count")?
        == 1;

    if existing_tables > 0 && !has_migrations {
        return Err(DbErr::Custom(format!(
            "unsupported pre-migration FIN database; recreate {}",
            database_path.display()
        )));
    }

    if !has_migrations {
        let transaction = db.begin().await?;
        for sql in INITIAL_SCHEMA {
            transaction.execute_unprepared(sql).await?;
        }
        transaction
            .execute_raw(Statement::from_sql_and_values(
                DatabaseBackend::Sqlite,
                "INSERT INTO fin_schema_migrations (version) VALUES (?)",
                [SCHEMA_VERSION.into()],
            ))
            .await?;
        transaction.commit().await?;
        return Ok(());
    }

    let version = db
        .query_one_raw(Statement::from_string(
            DatabaseBackend::Sqlite,
            "SELECT COALESCE(MAX(version), 0) AS version FROM fin_schema_migrations",
        ))
        .await?
        .ok_or_else(|| DbErr::Custom("failed to read schema version".to_string()))?
        .try_get::<i64>("", "version")?;

    if version != SCHEMA_VERSION {
        return Err(DbErr::Custom(format!(
            "unsupported FIN schema version {version}; expected {SCHEMA_VERSION}"
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::migrate;
    use sea_orm::{ConnectionTrait, Database, DatabaseBackend, Statement};
    use std::path::Path;

    #[tokio::test]
    async fn creates_schema_and_is_idempotent() {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        migrate(&db, Path::new("test.db")).await.unwrap();
        migrate(&db, Path::new("test.db")).await.unwrap();

        let version = db
            .query_one_raw(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT MAX(version) AS version FROM fin_schema_migrations",
            ))
            .await
            .unwrap()
            .unwrap()
            .try_get::<i64>("", "version")
            .unwrap();
        assert_eq!(version, 1);
    }

    #[tokio::test]
    async fn rejects_a_legacy_schema() {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        db.execute_raw(Statement::from_string(
            DatabaseBackend::Sqlite,
            "CREATE TABLE plans (id INTEGER PRIMARY KEY)",
        ))
        .await
        .unwrap();

        let error = migrate(&db, Path::new("sandbox.db")).await.unwrap_err();
        assert!(error.to_string().contains("pre-migration"));
        assert!(error.to_string().contains("sandbox.db"));
    }
}
