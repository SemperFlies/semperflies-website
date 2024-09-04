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
        DataResponse::error(&self.public_message(), Some(self.status_code()))
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
    pub fn error(message: impl ToString, code: Option<StatusCode>) -> DataApiReturn {
        (
            code.unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(Self {
                status: Status::Error,
                message: message.to_string(),
            }),
        )
    }

    pub fn success(message: impl ToString) -> DataApiReturn {
        (
            StatusCode::OK,
            Json(Self {
                status: Status::Success,
                message: message.to_string(),
            }),
        )
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
