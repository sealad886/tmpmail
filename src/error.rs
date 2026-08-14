use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppError {
    code: &'static str,
    message: String,
    exit_code: u8,
    retryable: bool,
}

impl AppError {
    pub fn new(code: &'static str, message: impl Into<String>, exit_code: u8) -> Self {
        Self {
            code,
            message: message.into(),
            exit_code,
            retryable: false,
        }
    }

    pub fn usage(message: impl Into<String>) -> Self {
        Self::new("usage", message, 2)
    }

    pub fn state(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(code, message, 3)
    }

    pub fn code(&self) -> &'static str {
        self.code
    }

    pub fn exit_code(&self) -> u8 {
        self.exit_code
    }

    pub fn retryable(&self) -> bool {
        self.retryable
    }

    pub fn with_retryable(mut self, retryable: bool) -> Self {
        self.retryable = retryable;
        self
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for AppError {}
