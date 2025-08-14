use crate::utils::models::OperatorValidationError;
use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use serde_json::{Value, json};
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone)]
pub enum ErrorType {
    InvalidImsiError,
    DuplicateImsiError,
    InvalidMsisdnError,
    DuplicateMsisdnError,
    InvalidTadigError,
    DuplicateTadigError,
    InvalidCountry,
    FieldValidationError,
    NotFound,
    BorderCountryNotFound,
    InternalError,
}

impl Display for ErrorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            ErrorType::InvalidImsiError => "InvalidImsiError",
            ErrorType::DuplicateImsiError => "DuplicateImsiError",
            ErrorType::InvalidMsisdnError => "InvalidMsisdnError",
            ErrorType::DuplicateMsisdnError => "DuplicateMsisdnError",
            ErrorType::InvalidTadigError => "InvalidTadigError",
            ErrorType::DuplicateTadigError => "DuplicateTadigError",
            ErrorType::InvalidCountry => "InvalidCountry",
            ErrorType::FieldValidationError => "FieldValidationError",
            ErrorType::NotFound => "NotFound",
            ErrorType::BorderCountryNotFound => "BorderCountryNotFound",
            ErrorType::InternalError => "InternalError",
        };
        write!(f, "{}", s)
    }
}

struct ErrorInfo {
    status: StatusCode,
    error_type: ErrorType,
    message: String,
    field: String,
    expected: Option<String>,
    received: Option<String>,
}

impl OperatorValidationError {
    fn into_info(self) -> ErrorInfo {
        match self {
            OperatorValidationError::InvalidImsiError {
                field,
                received,
                expected,
            } => ErrorInfo {
                status: StatusCode::BAD_REQUEST,
                error_type: ErrorType::InvalidImsiError,
                message: format!("{} must contain only digits", field),
                field,
                expected: Some(expected),
                received: Some(received),
            },
            OperatorValidationError::DuplicateImsiError { field, received } => ErrorInfo {
                status: StatusCode::CONFLICT,
                error_type: ErrorType::DuplicateImsiError,
                message: format!("IMSI prefix '{}' already exists", received),
                field,
                expected: None,
                received: Some(received),
            },
            OperatorValidationError::InvalidMsisdnError {
                field,
                received,
                expected,
            } => ErrorInfo {
                status: StatusCode::BAD_REQUEST,
                error_type: ErrorType::InvalidMsisdnError,
                message: format!("{} must contain only digits", field),
                field,
                expected: Some(expected),
                received: Some(received),
            },
            OperatorValidationError::DuplicateMsisdnError { field, received } => ErrorInfo {
                status: StatusCode::CONFLICT,
                error_type: ErrorType::DuplicateMsisdnError,
                message: format!("MSISDN prefix '{}' already exists", received),
                field,
                expected: None,
                received: Some(received),
            },
            OperatorValidationError::InvalidTadigError {
                field,
                received,
                expected,
            } => ErrorInfo {
                status: StatusCode::BAD_REQUEST,
                error_type: ErrorType::InvalidTadigError,
                message: format!("Invalid TADIG format for '{}'", received),
                field,
                expected: Some(expected),
                received: Some(received),
            },
            OperatorValidationError::DuplicateTadigError { field, received } => ErrorInfo {
                status: StatusCode::CONFLICT,
                error_type: ErrorType::DuplicateTadigError,
                message: format!("TADIG '{}' already exists", received),
                field,
                expected: None,
                received: Some(received),
            },
            OperatorValidationError::InvalidCountry {
                field,
                received,
                expected,
            } => ErrorInfo {
                status: StatusCode::NOT_FOUND,
                error_type: ErrorType::InvalidCountry,
                message: "Invalid country".to_string(),
                field,
                expected: Some(expected),
                received: Some(received),
            },
            OperatorValidationError::FieldValidationError {
                field,
                message,
                received,
            } => {
                let status = if message.contains("already exists") {
                    StatusCode::CONFLICT
                } else {
                    StatusCode::BAD_REQUEST
                };
                ErrorInfo {
                    status,
                    error_type: ErrorType::FieldValidationError,
                    message,
                    field,
                    expected: None,
                    received,
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum ErrorResponse {
    Validation(OperatorValidationError),
    NotFound {
        field: String,
        received: String,
        expected: String,
    },
    BorderCountryNotFound {
        field: String,
        received: String,
        expected: String,
    },
    InternalError,
}

impl ErrorResponse {
    fn into_info(self) -> ErrorInfo {
        match self {
            ErrorResponse::Validation(e) => e.into_info(),
            ErrorResponse::NotFound {
                field,
                received,
                expected,
            } => ErrorInfo {
                status: StatusCode::NOT_FOUND,
                error_type: ErrorType::NotFound,
                message: format!("{} not found", field.to_uppercase()),
                field,
                expected: Some(expected),
                received: Some(received),
            },
            ErrorResponse::BorderCountryNotFound {
                field,
                received,
                expected,
            } => ErrorInfo {
                status: StatusCode::NOT_FOUND,
                error_type: ErrorType::BorderCountryNotFound,
                message: format!("Bordering countries for {} not found", field.to_uppercase()),
                field,
                expected: Some(expected),
                received: Some(received),
            },
            ErrorResponse::InternalError => ErrorInfo {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                error_type: ErrorType::InternalError,
                message: "Internal server error".to_string(),
                field: "internal".to_string(),
                expected: None,
                received: None,
            },
        }
    }

    fn into_http(self) -> HttpResponse {
        let info = self.into_info();
        let mut body = json!({
            "error_type": info.error_type.to_string(),
            "error":      info.message,
            "field":      info.field,
        });
        if let Some(exp) = info.expected {
            body["expected"] = Value::String(exp);
        }
        if let Some(rec) = info.received {
            body["received"] = Value::String(rec);
        }
        HttpResponse::build(info.status).json(body)
    }
}

impl Display for ErrorResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ErrorResponse::Validation(e) => write!(f, "{:?}", e),
            ErrorResponse::NotFound { field, .. } => write!(f, "{} not found", field),
            ErrorResponse::BorderCountryNotFound { field, .. } => {
                write!(f, "Bordering countries for {} not found", field)
            }
            ErrorResponse::InternalError => write!(f, "Internal server error"),
        }
    }
}

impl ResponseError for ErrorResponse {
    fn status_code(&self) -> StatusCode {
        self.clone().into_info().status
    }

    fn error_response(&self) -> HttpResponse {
        self.clone().into_http()
    }
}

impl From<OperatorValidationError> for ErrorResponse {
    fn from(e: OperatorValidationError) -> Self {
        ErrorResponse::Validation(e)
    }
}
