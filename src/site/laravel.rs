use crate::{
    env::extract_env_var,
    error::{AppError, AppResult},
    types::Credentials,
};

pub fn get_credentials(folder: &std::path::Path) -> AppResult<Credentials> {
    if let Ok(contents) = std::fs::read_to_string(folder.join(".env")) {
        let db_name = extract_env_var(&contents, "DB_DATABASE");
        let username = extract_env_var(&contents, "DB_USERNAME");
        let password = extract_env_var(&contents, "DB_PASSWORD");
        if let (Some(database), Some(username), Some(password)) = (db_name, username, password) {
            return Ok(Credentials {
                database,
                username,
                password,
            });
        }
    }

    Err(AppError::MissingDatabaseCredentials)
}
