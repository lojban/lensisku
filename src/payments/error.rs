use std::fmt;

#[derive(Debug)]
pub enum PaymentError {
    Provider(String),
    Database(String),
    Transaction(String),
    NotFound(String),
}

impl std::error::Error for PaymentError {}

impl fmt::Display for PaymentError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PaymentError::Provider(msg) => write!(f, "Payment provider error: {}", msg),
            PaymentError::Database(msg) => write!(f, "Database error: {}", msg),
            PaymentError::Transaction(msg) => write!(f, "Transaction error: {}", msg),
            PaymentError::NotFound(msg) => write!(f, "Not found error: {}", msg),
        }
    }
}

impl From<Box<dyn std::error::Error>> for PaymentError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        PaymentError::Provider(err.to_string())
    }
}

/// Keeps the old `Box<dyn Error>` text: `detail()` returns this error's `Display`.
impl From<PaymentError> for crate::error::AppError {
    fn from(err: PaymentError) -> Self {
        crate::error::AppError::Internal(err.to_string())
    }
}
