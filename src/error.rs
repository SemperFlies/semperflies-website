use axum::Json;
use reqwest::StatusCode;
use serde::Serialize;

pub type DataApiReturn = (StatusCode, Json<DataResponse>);
pub trait InternalError {
    fn status_code(&self) -> StatusCode;
    fn public_message(&self) -> String;
    /// All errors should implement data response. Data responses are coerced into
    /// HTML and are customer facing
    fn into_data_api_return(&self) -> DataApiReturn {
        (
            self.status_code(),
            Json(DataResponse::error(&self.public_message())),
        )
    }
}

#[derive(Debug, Serialize)]
pub struct DataResponse {
    status: Status,
    message: String,
}

#[derive(Debug, Serialize)]
enum Status {
    Error,
    Success,
}

impl Into<&'static str> for Status {
    fn into(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Success => "success",
        }
    }
}

impl DataResponse {
    pub fn error(message: &str) -> Self {
        Self {
            status: Status::Error,
            message: message.to_string(),
        }
    }

    pub fn success(message: &str) -> Self {
        Self {
            status: Status::Success,
            message: message.to_string(),
        }
    }

    pub fn from_internal_error(err: &impl InternalError) -> Self {
        Self::error(&err.public_message())
    }
}

#[allow(unused_must_use)]
pub fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e);
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}
