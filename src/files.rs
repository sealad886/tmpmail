use crate::error::AppError;
use rand::distr::{Alphanumeric, SampleString};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn safe_basename(value: &str) -> Result<String, AppError> {
    if value.is_empty()
        || value == "."
        || value == ".."
        || value.contains('/')
        || value.contains('\\')
        || value.chars().any(|character| character.is_control())
    {
        return Err(AppError::new("unsafe_path", "unsafe filename refused", 9));
    }
    let stem = value.split('.').next().unwrap_or(value);
    let reserved = ["CON", "PRN", "AUX", "NUL", "COM1", "LPT1"];
    if reserved.iter().any(|name| stem.eq_ignore_ascii_case(name)) {
        return Err(AppError::new("unsafe_path", "reserved filename refused", 9));
    }
    Ok(value.to_owned())
}

pub fn temporary_sibling(path: &Path) -> Result<(PathBuf, std::fs::File), AppError> {
    for _ in 0..8 {
        let suffix = Alphanumeric.sample_string(&mut rand::rng(), 12);
        let temporary = path.with_extension(format!("tmpmail-{suffix}.part"));
        let mut options = OpenOptions::new();
        options.create_new(true).write(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        match options.open(&temporary) {
            Ok(file) => return Ok((temporary, file)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(AppError::new(
                    "unsafe_path",
                    format!("could not create output: {error}"),
                    9,
                ));
            }
        }
    }
    Err(AppError::new(
        "unsafe_path",
        "could not allocate temporary output",
        9,
    ))
}

pub fn atomic_write(path: &Path, bytes: &[u8], replace: bool) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| AppError::state("file_write", e.to_string()))?;
    }
    if fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        return Err(AppError::new(
            "unsafe_path",
            "refusing to replace a symbolic link",
            9,
        ));
    }
    if path.exists() && !replace {
        return Err(AppError::new("unsafe_path", "destination exists", 9));
    }
    let (temporary, mut file) = temporary_sibling(path)?;
    let result = file.write_all(bytes).and_then(|_| file.sync_all());
    if let Err(error) = result {
        let _ = fs::remove_file(&temporary);
        return Err(AppError::state("file_write", error.to_string()));
    }
    publish_temporary(&temporary, path, replace)?;
    Ok(())
}

pub fn publish_temporary(temporary: &Path, path: &Path, replace: bool) -> Result<(), AppError> {
    if replace {
        fs::rename(temporary, path).map_err(|error| {
            let _ = fs::remove_file(temporary);
            AppError::state("file_write", error.to_string())
        })?;
    } else {
        fs::hard_link(temporary, path).map_err(|error| {
            let _ = fs::remove_file(temporary);
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                AppError::new("unsafe_path", "destination exists", 9)
            } else {
                AppError::state("file_write", error.to_string())
            }
        })?;
        fs::remove_file(temporary)
            .map_err(|error| AppError::state("file_write", error.to_string()))?;
    }
    if let Some(parent) = path.parent()
        && let Ok(directory) = fs::File::open(parent)
    {
        let _ = directory.sync_all();
    }
    Ok(())
}
