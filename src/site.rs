use std::{
    fs::File,
    io::Write,
    marker::PhantomData,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use crate::{
    error::{AppError, AppResult},
    types::Credentials,
};
mod laravel;
mod wordpress;

#[derive(Debug)]
pub enum SiteType {
    Wordpress,
    Laravel,
    Django,
    Rails,
    Express,
    Flask,
    Drupal,
    Magento,
}

pub struct NotDetected;
pub struct Detected;

pub struct NoCredentials;
pub struct GotCredentials;

pub struct DbBackedUp;
pub struct DbNotBackedUp;

pub struct FilesBackedUp;
pub struct FilesNotBackedUp;

pub struct Site<DetectionState, CredentialsState, DbBackupState, FileBackupState> {
    detect_state: PhantomData<DetectionState>,
    credentials_state: PhantomData<CredentialsState>,
    db_backup_state: PhantomData<DbBackupState>,
    files_backup_state: PhantomData<FileBackupState>,

    path: PathBuf,
    temp: PathBuf,
    site_type: Option<SiteType>,
    credentials: Option<Credentials>,
}

impl Site<NotDetected, NoCredentials, DbNotBackedUp, FilesNotBackedUp> {
    pub fn new(
        path: PathBuf,
        temp: PathBuf,
    ) -> Site<NotDetected, NoCredentials, DbNotBackedUp, FilesNotBackedUp> {
        Site {
            detect_state: PhantomData,
            credentials_state: PhantomData,
            db_backup_state: PhantomData,
            files_backup_state: PhantomData,

            path,
            temp,
            site_type: None,
            credentials: None,
        }
    }
}

impl<U> Site<NotDetected, U, DbNotBackedUp, FilesNotBackedUp> {
    pub fn detect(self) -> AppResult<Site<Detected, U, DbNotBackedUp, FilesNotBackedUp>> {
        let site_type = detect_site_type(&self.path)?;
        Ok(Site {
            detect_state: PhantomData,
            credentials_state: PhantomData,
            db_backup_state: PhantomData,
            files_backup_state: PhantomData,

            path: self.path,
            temp: self.temp,
            site_type: Some(site_type),
            credentials: self.credentials,
        })
    }
}

impl<W> Site<Detected, NoCredentials, DbNotBackedUp, W> {
    pub fn get_credentials(self) -> AppResult<Site<Detected, GotCredentials, DbNotBackedUp, W>> {
        let creds = match self.site_type {
            Some(SiteType::Laravel) => laravel::get_credentials(&self.path),
            Some(SiteType::Wordpress) => wordpress::get_credentials(&self.path),
            _ => Err(AppError::MissingDatabaseCredentials),
        }?;
        Ok(Site {
            detect_state: PhantomData,
            credentials_state: PhantomData,
            db_backup_state: PhantomData,
            files_backup_state: PhantomData,

            path: self.path,
            temp: self.temp,
            site_type: self.site_type,
            credentials: Some(creds),
        })
    }
}

impl<W> Site<Detected, GotCredentials, DbNotBackedUp, W> {
    pub fn db_backup(self) -> AppResult<Site<Detected, GotCredentials, DbBackedUp, W>> {
        // Create the output path with a filename
        let backup_dir = self.temp.join("forge_backup");
        std::fs::create_dir_all(&backup_dir)?;
        let out_path = backup_dir.join("backup.sql.gz");
        let creds = self
            .credentials
            .as_ref()
            .expect("Credentials will always be some in GotCredentials generic type");

        // Step 1. Run mysqldump and capture output

        let dump = Command::new("mysqldump")
            .arg("-u")
            .arg(&creds.username)
            .arg("--no-tablespaces")
            .arg(&creds.database)
            .env("MYSQL_PWD", &creds.password)
            .stdout(Stdio::piped())
            .spawn()?;

        let dump_output = dump.wait_with_output()?;

        // Step 2. Pipe the output of mysqldump into gzip
        let gzip = Command::new("gzip")
            .arg("-c9")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        {
            let mut stdin = gzip.stdin.as_ref().ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::Other, "Failed to open stdin")
            })?;
            stdin.write_all(&dump_output.stdout)?;
        }

        let gzip_output = gzip.wait_with_output()?;

        // Step 3. Write the gzipped output to a file
        _ = File::create(out_path)?.write_all(&gzip_output.stdout)?;

        Ok(Site {
            detect_state: PhantomData,
            credentials_state: PhantomData,
            db_backup_state: PhantomData,
            files_backup_state: PhantomData,
            path: self.path,
            temp: self.temp,
            site_type: self.site_type,
            credentials: self.credentials,
        })
    }
}
impl<V> Site<Detected, GotCredentials, V, FilesNotBackedUp> {
    pub fn files_backup(self) -> AppResult<Site<Detected, GotCredentials, V, FilesBackedUp>> {
        let backup_dir = self.temp.join("forge_backup");
        std::fs::create_dir_all(&backup_dir)?;
        let out_path = backup_dir.join("backup.files.tar.gz");
        let tar = Command::new("tar")
            .arg("-zcpvf")
            .arg(out_path)
            .arg(".")
            .current_dir(&self.path)
            .stdout(Stdio::piped())
            .output()?;

        if !tar.status.success() {
            return Err(AppError::BackupError(
                String::from_utf8(tar.stderr).expect("Unable to read stderr"),
            ));
        }

        Ok(Site {
            detect_state: PhantomData,
            credentials_state: PhantomData,
            db_backup_state: PhantomData,
            files_backup_state: PhantomData,
            path: self.path,
            temp: self.temp,
            site_type: self.site_type,
            credentials: self.credentials,
        })
    }
}

fn detect_site_type(folder: &Path) -> AppResult<SiteType> {
    // Check for Laravel
    if folder.join("artisan").exists()
        && folder.join("composer.json").exists()
        && folder.join("config").join("app.php").exists()
    {
        return Ok(SiteType::Laravel);
    }

    // Check for wordpress
    if folder.join("wp-config.php").exists()
        && folder.join("wp-load.php").exists()
        && folder.join("wp-content").exists()
    {
        return Ok(SiteType::Wordpress);
    }

    // Check for Rails
    if folder.join("config.ru").exists()
        && folder.join("Gemfile").exists()
        && folder.join("bin").join("rails").exists()
    {
        return Ok(SiteType::Rails);
    }

    // Check for Django
    if folder.join("manage.py").exists()
        && (folder.join("requirements.txt").exists() || folder.join("Pipfile").exists())
    {
        return Ok(SiteType::Django);
    }

    // Check for Express (Node.js)
    if (folder.join("app.js").exists() || folder.join("server.js").exists())
        && folder.join("package.json").exists()
    {
        return Ok(SiteType::Express);
    }

    // Check for Flask
    if (folder.join("app.py").exists() || folder.join("main.py").exists())
        && folder.join("requirements.txt").exists()
    {
        return Ok(SiteType::Flask);
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
        return Ok(SiteType::Drupal);
    }

    // Check for Magento
    if folder.join("index.php").exists()
        && folder.join("app").join("etc").join("env.php").exists()
        && folder.join("app").join("Mage.php").exists()
    {
        return Ok(SiteType::Magento);
    }

    Err(AppError::UnknownSite)
}
