use crate::error::AppError;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
}

pub fn clipboard_command() -> CommandSpec {
    #[cfg(target_os = "macos")]
    return CommandSpec {
        program: "pbcopy".to_owned(),
        args: Vec::new(),
    };
    #[cfg(target_os = "linux")]
    return CommandSpec {
        program: "xclip".to_owned(),
        args: vec!["-selection".to_owned(), "clipboard".to_owned()],
    };
    #[allow(unreachable_code)]
    CommandSpec {
        program: String::new(),
        args: Vec::new(),
    }
}

pub fn browser_command(path: &Path) -> CommandSpec {
    #[cfg(target_os = "macos")]
    return CommandSpec {
        program: "open".to_owned(),
        args: vec![path.display().to_string()],
    };
    #[cfg(target_os = "linux")]
    return CommandSpec {
        program: "xdg-open".to_owned(),
        args: vec![path.display().to_string()],
    };
    #[allow(unreachable_code)]
    CommandSpec {
        program: String::new(),
        args: Vec::new(),
    }
}

pub fn copy(text: &str) -> Result<(), AppError> {
    let spec = clipboard_command();
    let mut child = Command::new(&spec.program)
        .args(&spec.args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| AppError::new("clipboard_unavailable", "clipboard command unavailable", 3))?;
    child
        .stdin
        .take()
        .ok_or_else(|| AppError::new("clipboard_unavailable", "clipboard input unavailable", 3))?
        .write_all(text.as_bytes())
        .map_err(|_| AppError::new("clipboard_failure", "could not copy address", 3))?;
    let status = child
        .wait()
        .map_err(|_| AppError::new("clipboard_failure", "clipboard command failed", 3))?;
    if status.success() {
        Ok(())
    } else {
        Err(AppError::new(
            "clipboard_failure",
            "clipboard command failed",
            3,
        ))
    }
}

pub fn open_browser(path: &Path) -> Result<(), AppError> {
    let spec = browser_command(path);
    let status = Command::new(&spec.program)
        .args(&spec.args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|_| AppError::new("browser_unavailable", "browser command unavailable", 3))?;
    if status.success() {
        Ok(())
    } else {
        Err(AppError::new(
            "browser_failure",
            "browser command failed",
            3,
        ))
    }
}
