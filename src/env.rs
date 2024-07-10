use regex::Regex;

pub fn extract_env_var(contents: &str, var_name: &str) -> Option<String> {
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
