use regex::Regex;

use crate::{
    error::{AppError, AppResult},
    types::Credentials,
};

pub fn get_credentials(folder: &std::path::Path) -> AppResult<Credentials> {
    if let Ok(contents) = std::fs::read_to_string(folder.join("wp-config.php")) {
        let db_name = extract_wp_var(&contents, "DB_NAME");
        let username = extract_wp_var(&contents, "DB_USER");
        let password = extract_wp_var(&contents, "DB_PASSWORD");
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

fn extract_wp_var(contents: &str, var_name: &str) -> Option<String> {
    let re = Regex::new(&format!(
        r#"(?m)^[\t ]*define\([\t ]*['"]{}['"][\t ]*,[\t ]*(['"])(.*?)(\1)"#,
        regex::escape(var_name)
    ))
    .ok()?;

    re.captures(contents)
        .and_then(|cap| cap.get(2).map(|v| v.as_str().trim().to_string()))
}
