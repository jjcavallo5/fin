use crate::logging;
use sea_orm::{Database, DatabaseConnection};

fn create_db() -> std::path::PathBuf {
    let db_path = dirs::home_dir().unwrap().join(".fin/plans.db");
    let exists = std::fs::exists(&db_path).unwrap_or_else(|_| {
        logging::error("failed to create database");
        std::process::exit(1)
    });
    if !exists {
        std::fs::File::create_new(&db_path).unwrap_or_else(|_| {
            logging::error("failed to create database");
            std::process::exit(1)
        });
    }
    return db_path;
}

pub async fn get_db() -> DatabaseConnection {
    let db_path = create_db();
    let db = Database::connect(format!("sqlite://{}", db_path.display()))
        .await
        .unwrap();
    db.get_schema_registry("fin::entity::*")
        .sync(&db)
        .await
        .unwrap();
    return db;
}
