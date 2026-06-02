use sea_orm::{Database, DatabaseConnection};

pub async fn get_db() -> DatabaseConnection {
    let db_path = dirs::home_dir().unwrap().join(".fin/plans.db");
    println!("{}", db_path.display());
    let db = Database::connect(format!("sqlite://{}", db_path.display()))
        .await
        .unwrap();
    db.get_schema_registry("fin::entity::*")
        .sync(&db)
        .await
        .unwrap();
    return db;
}
