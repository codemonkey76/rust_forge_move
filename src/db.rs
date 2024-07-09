use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
    process::{Command, Stdio},
};

use crate::error::AppResult;

#[derive(Debug)]
pub struct Credentials {
    pub database: String,
    pub username: String,
    pub password: String,
}

pub fn backup(credentials: Credentials, output_dir: &Path) -> AppResult<()> {
    // Create the output path with a filename
    let backup_dir = output_dir.join("forge_backup");
    fs::create_dir_all(&backup_dir)?;
    let out_path = backup_dir.join("backup.sql.gz");

    // Step 1. Run mysqldump and capture output
    let dump = Command::new("mysqldump")
        .arg("-u")
        .arg(&credentials.username)
        .arg("--no-tablespaces")
        .arg(&credentials.database)
        .env("MYSQL_PWD", &credentials.password)
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
        let mut stdin = gzip
            .stdin
            .as_ref()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to open stdin"))?;
        stdin.write_all(&dump_output.stdout)?;
    }

    let gzip_output = gzip.wait_with_output()?;

    // Step 3. Write the gzipped output to a file
    _ = File::create(out_path)?.write_all(&gzip_output.stdout)?;

    Ok(())
}
