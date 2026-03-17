use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Web3CliError {
    #[error("{message}")]
    Transaction { message: String },

    #[error("{0}")]
    Auth(String),

    #[error("{0}")]
    Validation(String),

    #[error("{0}")]
    Network(String),

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub code: &'static str,
    pub message: String,
    pub category: &'static str,
}

impl Web3CliError {
    pub const EXIT_CODE_TRANSACTION: u8 = 1;
    pub const EXIT_CODE_AUTH: u8 = 2;
    pub const EXIT_CODE_VALIDATION: u8 = 3;
    pub const EXIT_CODE_NETWORK: u8 = 4;
    pub const EXIT_CODE_INTERNAL: u8 = 5;

    pub fn exit_code(&self) -> u8 {
        match self {
            Self::Transaction { .. } => Self::EXIT_CODE_TRANSACTION,
            Self::Auth(_) => Self::EXIT_CODE_AUTH,
            Self::Validation(_) => Self::EXIT_CODE_VALIDATION,
            Self::Network(_) => Self::EXIT_CODE_NETWORK,
            Self::Internal(_) => Self::EXIT_CODE_INTERNAL,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Transaction { .. } => "TRANSACTION_ERROR",
            Self::Auth(_) => "AUTH_ERROR",
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::Network(_) => "NETWORK_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }

    pub fn category(&self) -> &'static str {
        match self {
            Self::Transaction { .. } => "transaction",
            Self::Auth(_) => "auth",
            Self::Validation(_) => "validation",
            Self::Network(_) => "network",
            Self::Internal(_) => "internal",
        }
    }

    pub fn to_error_response(&self) -> ErrorResponse {
        ErrorResponse {
            code: self.error_code(),
            message: self.to_string(),
            category: self.category(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_codes_are_distinct() {
        let codes = [
            Web3CliError::EXIT_CODE_TRANSACTION,
            Web3CliError::EXIT_CODE_AUTH,
            Web3CliError::EXIT_CODE_VALIDATION,
            Web3CliError::EXIT_CODE_NETWORK,
            Web3CliError::EXIT_CODE_INTERNAL,
        ];
        let unique: std::collections::HashSet<u8> = codes.iter().copied().collect();
        assert_eq!(unique.len(), codes.len());
    }

    #[test]
    fn exit_code_mapping() {
        assert_eq!(
            Web3CliError::Transaction {
                message: "fail".into()
            }
            .exit_code(),
            1
        );
        assert_eq!(Web3CliError::Auth("fail".into()).exit_code(), 2);
        assert_eq!(Web3CliError::Validation("fail".into()).exit_code(), 3);
        assert_eq!(Web3CliError::Network("fail".into()).exit_code(), 4);
        assert_eq!(
            Web3CliError::Internal(anyhow::anyhow!("fail")).exit_code(),
            5
        );
    }

    #[test]
    fn error_response_serialization() {
        let err = Web3CliError::Validation("bad address".into());
        let resp = err.to_error_response();
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["code"], "VALIDATION_ERROR");
        assert_eq!(json["message"], "bad address");
        assert_eq!(json["category"], "validation");
    }
}
