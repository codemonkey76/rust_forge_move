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
        if let (Some(database), Some(username), Some(password)) = (&db_name, &username, &password) {
            return Ok(Credentials {
                database: database.to_string(),
                username: username.to_string(),
                password: password.to_string(),
            });
        } else {
            eprintln!("Unable to read required credentials (db_name, db_user, db_password): ({:?}, {:?}, {:?})", db_name, username, password);
        }
    } else {
        eprintln!("Unable to read file: {:?}", folder.join("wp-config.php"));
    }
    Err(AppError::MissingDatabaseCredentials)
}

fn extract_wp_var(contents: &str, var_name: &str) -> Option<String> {
    let re = Regex::new(&format!(
        r#"(?m)^[\t ]*define\([\t ]*['"]{}['"][\t ]*,[\t ]*['"](.*?)['"][\t ]*\);"#,
        regex::escape(var_name)
    ))
    .ok()?;

    re.captures(contents)
        .and_then(|cap| cap.get(1).map(|v| v.as_str().trim().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_wp_var() {
        let contents = r#"
/** The name of the database for WordPress */
define( 'DB_NAME', 'testing' );

/** Database username */
define( 'DB_USER', 'username' );

/** Database password */
define( 'DB_PASSWORD', 'secret123' );
"#;
        assert_eq!(
            extract_wp_var(contents, "DB_NAME"),
            Some("testing".to_string())
        );
        assert_eq!(
            extract_wp_var(contents, "DB_USER"),
            Some("username".to_string())
        );
        assert_eq!(
            extract_wp_var(contents, "DB_PASSWORD"),
            Some("secret123".to_string())
        );
    }
}
