use crate::daemon::encryption::{self, SALT_LEN};
use crate::{entity, environment, logging, migration};
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, EntityTrait, Set};

fn database_path(home_dir: &std::path::Path) -> std::path::PathBuf {
    home_dir.join(".fin").join(environment::DATABASE_FILENAME)
}

fn create_db() -> std::path::PathBuf {
    let home_dir = dirs::home_dir().unwrap_or_else(|| {
        logging::error("failed to locate home directory");
        std::process::exit(1)
    });
    let db_path = database_path(&home_dir);
    let fin_dir = db_path.parent().unwrap();
    std::fs::create_dir_all(&fin_dir).unwrap_or_else(|_| {
        logging::error("failed to create .fin directory");
        std::process::exit(1)
    });

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
    migration::migrate(&db, &db_path)
        .await
        .unwrap_or_else(|error| {
            logging::error(&format!("failed to initialize database: {error}"));
            std::process::exit(1)
        });
    get_db_salt(&db).await;
    return db;
}

#[cfg(test)]
mod tests {
    use super::database_path;
    use crate::environment;
    use std::path::Path;

    #[test]
    fn builds_database_path_for_active_environment() {
        assert_eq!(
            database_path(Path::new("/test-home")),
            Path::new("/test-home")
                .join(".fin")
                .join(environment::DATABASE_FILENAME)
        );
    }
}
