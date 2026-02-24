#[derive(Debug)]
pub(crate) enum EmailError {
    SendError(String),
    ConfigError(String),
}

impl std::fmt::Display for EmailError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmailError::SendError(msg) => write!(f, "Send error: {}", msg),
            EmailError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

#[derive(Debug)]
pub enum PasswordHashError {
    BcryptError(bcrypt::BcryptError),
    InvalidHashFormat,
}

impl std::fmt::Display for PasswordHashError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PasswordHashError::BcryptError(e) => write!(f, "Bcrypt error: {}", e),
            PasswordHashError::InvalidHashFormat => write!(f, "Invalid hash format"),
        }
    }
}

impl std::error::Error for PasswordHashError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PasswordHashError::BcryptError(e) => Some(e),
            PasswordHashError::InvalidHashFormat => None,
        }
    }
}

impl From<bcrypt::BcryptError> for PasswordHashError {
    fn from(error: bcrypt::BcryptError) -> Self {
        PasswordHashError::BcryptError(error)
    }
}
