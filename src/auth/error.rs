use reqwest::StatusCode;

use crate::error::{error_chain_fmt, InternalError};
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

pub type AuthResult<T> = Result<T, AuthError>;
#[derive(thiserror::Error)]
pub enum AuthError {
    #[error(transparent)]
    Undefined(#[from] anyhow::Error),
    JsonWebToken(#[from] jsonwebtoken::errors::Error),
    Uuid(#[from] uuid::Error),
    NotLoggedIn,
}

impl Debug for AuthError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        error_chain_fmt(self, f)
    }
}

impl Display for AuthError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let display = match self {
            Self::Undefined(err) => err.to_string(),
            Self::JsonWebToken(err) => err.to_string(),
            Self::Uuid(err) => err.to_string(),
            Self::NotLoggedIn => "Not Logged In".to_string(),
        };
        writeln!(f, "{}", display)
    }
}

impl InternalError for AuthError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::NotLoggedIn => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn public_message(&self) -> String {
        match self {
            Self::NotLoggedIn => "You are not logged in".to_string(),
            Self::JsonWebToken(err) => format!("A web token error occurred: {:?}", err),
            Self::Undefined(err) => format!("An undefined error occurred: {:?}", err),
            Self::Uuid(err) => format!("A uuid error occurred: {:?}", err),
        }
    }
}
