use std::{
    fs, io,
    path::{Path, PathBuf},
};

use args::Args;
use clap::Parser;
use dirs::home_dir;
use regex::Regex;
mod db;

mod args;
mod error;
use error::{AppError, AppResult};

fn main() -> AppResult<()> {
    let args = Args::parse();
    let site = detect_site_type(&args.dir)?;
    let creds = get_database_credentials(&args.dir, site)?;
    let home_dir = get_home_dir()?;
    db::backup(creds, &home_dir)?;

    Ok(())
}

#[derive(Debug)]
enum Site {
    Wordpress,
    Laravel,
    Django,
    Rails,
    Express,
    Flask,
    Drupal,
    Magento,
}

fn get_home_dir() -> AppResult<PathBuf> {
    Ok(home_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Home directory not found"))?)
}

fn detect_site_type(folder_path: &str) -> AppResult<Site> {
    let folder = Path::new(folder_path);

    // Check for Laravel
    if folder.join("artisan").exists()
        && folder.join("composer.json").exists()
        && folder.join("config").join("app.php").exists()
    {
        return Ok(Site::Laravel);
    }

    // Check for wordpress
    if folder.join("wp-config.php").exists()
        && folder.join("wp-load.php").exists()
        && folder.join("wp-content").exists()
    {
        return Ok(Site::Wordpress);
    }

    // Check for Rails
    if folder.join("config.ru").exists()
        && folder.join("Gemfile").exists()
        && folder.join("bin").join("rails").exists()
    {
        return Ok(Site::Rails);
    }

    // Check for Django
    if folder.join("manage.py").exists()
        && (folder.join("requirements.txt").exists() || folder.join("Pipfile").exists())
    {
        return Ok(Site::Django);
    }

    // Check for Express (Node.js)
    if (folder.join("app.js").exists() || folder.join("server.js").exists())
        && folder.join("package.json").exists()
    {
        return Ok(Site::Express);
    }

    // Check for Flask
    if (folder.join("app.py").exists() || folder.join("main.py").exists())
        && folder.join("requirements.txt").exists()
    {
        return Ok(Site::Flask);
    }

    // Check for Drupal
    if folder.join("index.php").exists()
        && folder
            .join("core")
            .join("includes")
            .join("bootstrap.inc")
            .exists()
        && folder
            .join("sites")
            .join("default")
            .join("settings.php")
            .exists()
    {
        return Ok(Site::Drupal);
    }

    // Check for Magento
    if folder.join("index.php").exists()
        && folder.join("app").join("etc").join("env.php").exists()
        && folder.join("app").join("Mage.php").exists()
    {
        return Ok(Site::Magento);
    }

    Err(AppError::UnknownSite)
}

fn get_database_credentials(folder_path: &str, site_type: Site) -> AppResult<db::Credentials> {
    let folder = Path::new(folder_path);

    match site_type {
        Site::Laravel => {
            // Check for .env file
            if let Ok(contents) = fs::read_to_string(folder.join(".env")) {
                let db_name = extract_env_var(&contents, "DB_DATABASE");
                let username = extract_env_var(&contents, "DB_USERNAME");
                let password = extract_env_var(&contents, "DB_PASSWORD");
                if let (Some(database), Some(username), Some(password)) =
                    (db_name, username, password)
                {
                    return Ok(db::Credentials {
                        database,
                        username,
                        password,
                    });
                }
            }
        }
        Site::Wordpress => {
            if let Ok(contents) = fs::read_to_string(folder.join("wp-config.php")) {
                let db_name = extract_wp_var(&contents, "DB_NAME");
                let username = extract_wp_var(&contents, "DB_USER");
                let password = extract_wp_var(&contents, "DB_PASS");
                if let (Some(database), Some(username), Some(password)) =
                    (db_name, username, password)
                {
                    return Ok(db::Credentials {
                        database,
                        username,
                        password,
                    });
                }
            }
        }

        _ => {}
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

fn extract_env_var(contents: &str, var_name: &str) -> Option<String> {
    let re = Regex::new(&format!(
        r#"(?m)^[\t ]*{}[\t ]*=[\t ]*("(?:[^"]*)"|[^"\n]*)[\t ]*$"#,
        regex::escape(var_name)
    ))
    .ok()?;

    re.captures(contents).and_then(|cap| {
        cap.get(1)
            .map(|v| v.as_str().trim_matches('"').trim().to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_env_var() {
        let contents = r#"
LOG_CHANNEL=stack
LOG_STACK=single
LOG_DEPRECATIONS_CHANNEL=null
LOG_LEVEL=debug

DB_CONNECTION=mariadb
DB_HOST=127.0.0.1
DB_PORT=3306
DB_DATABASE=testing
DB_USERNAME=root
DB_PASSWORD=""

SESSION_DRIVER=database
SESSION_LIFETIME=120
SESSION_ENCRYPT=false
SESSION_PATH=/
SESSION_DOMAIN=null
"#;
        assert_eq!(
            extract_env_var(contents, "DB_DATABASE"),
            Some("testing".to_string())
        );
        assert_eq!(
            extract_env_var(contents, "DB_USERNAME"),
            Some("root".to_string())
        );
        assert_eq!(
            extract_env_var(contents, "DB_PASSWORD"),
            Some("".to_string())
        );
        assert_eq!(
            extract_env_var(contents, "DB_HOST"),
            Some("127.0.0.1".to_string())
        );
        assert_eq!(
            extract_env_var(contents, "DB_PORT"),
            Some("3306".to_string())
        );
    }
}
