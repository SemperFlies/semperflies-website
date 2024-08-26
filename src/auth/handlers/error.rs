use axum::extract::multipart::MultipartError;
use reqwest::StatusCode;

use crate::error::{error_chain_fmt, InternalError};
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

pub type UploadResult<T> = Result<T, UploadError>;
#[derive(thiserror::Error)]
pub enum UploadError {
    #[error(transparent)]
    Undefined(#[from] anyhow::Error),
    Chrono(#[from] chrono::ParseError),
    Serde(#[from] serde_json::error::Error),
    MultiPart(#[from] MultipartError),
    UserFacing(String),
}

impl Debug for UploadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        error_chain_fmt(self, f)
    }
}

impl Display for UploadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let display = match self {
            Self::Undefined(err) => err.to_string(),
            Self::Serde(err) => err.to_string(),
            Self::MultiPart(err) => err.to_string(),
            Self::UserFacing(err) => err.to_string(),
            Self::Chrono(err) => err.to_string(),
        };
        writeln!(f, "{}", display)
    }
}

impl InternalError for UploadError {
    fn status_code(&self) -> StatusCode {
        match self {
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn public_message(&self) -> String {
        match self {
            Self::UserFacing(err) => err.to_string(),
            Self::Undefined(err) => format!("An undefined error occurred: {:?}", err),
            Self::MultiPart(err) => format!("A multipart error occurred: {:?}", err),
            Self::Serde(err) => format!("A serde error occurred: {:?}", err),
            Self::Chrono(err) => format!("A chrono error occurred: {:?}", err),
        }
    }
}

impl UploadError {
    pub fn user_facing(str: &str) -> Self {
        Self::UserFacing(str.to_string())
    }
}
