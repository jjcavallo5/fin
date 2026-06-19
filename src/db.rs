use crate::daemon::encryption::{self, SALT_LEN};
use crate::{entity, logging};
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, EntityTrait, Set};

fn create_db() -> std::path::PathBuf {
    let fin_dir = dirs::home_dir().unwrap().join(".fin");
    std::fs::create_dir_all(&fin_dir).unwrap_or_else(|_| {
        logging::error("failed to create .fin directory");
        std::process::exit(1)
    });

    let db_path = fin_dir.join("plans.db");
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

pub async fn get_db_salt(db: &DatabaseConnection) -> [u8; SALT_LEN] {
    let existing_salt = entity::encryption::Entity::find()
        .one(db)
        .await
        .unwrap_or_else(|_| {
            logging::error("failed to query encryption salt");
            std::process::exit(1)
        });

    let salt = match existing_salt {
        Some(model) => model.salt,
        None => {
            let generated_salt = encryption::generate_db_salt();
            let encoded_salt = encryption::encode_hex(&generated_salt);
            let active_model = entity::encryption::ActiveModel {
                salt: Set(encoded_salt.clone()),
            };
            active_model.insert(db).await.unwrap_or_else(|_| {
                logging::error("failed to save encryption salt");
                std::process::exit(1)
            });
            encoded_salt
        }
    };

    let decoded_salt = encryption::decode_hex(&salt).unwrap_or_else(|_| {
        logging::error("stored encryption salt is invalid");
        std::process::exit(1)
    });

    let fixed_size_salt: [u8; SALT_LEN] = decoded_salt.try_into().unwrap();
    return fixed_size_salt;
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
    get_db_salt(&db).await;
    return db;
}
