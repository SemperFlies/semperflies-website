mod error;
pub mod upload;
use super::model::{LoginAdminSchema, TokenClaims};
use crate::{
    auth::{error::AuthError, ADMIN_CREDENTIALS},
    error::{DataApiReturn, InternalError},
    state::SharedState,
    AppState,
};
use anyhow::anyhow;
use axum::{
    extract::{Multipart, Path, State},
    http::{header, Response},
    response::IntoResponse,
    Form,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::json;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

pub async fn login_admin_handler(
    State(data): State<SharedState>,
    Form(body): Form<LoginAdminSchema>,
) -> Result<impl IntoResponse, DataApiReturn> {
    info!("Login request Body {:?}", body);

    if ADMIN_CREDENTIALS.password != body.password {
        data.write().await.admin_session_id = None;
        return Err(AuthError::NotLoggedIn.into_data_api_return());
    }

    let now = chrono::Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + chrono::Duration::minutes(60)).timestamp() as usize;
    let uuid = Uuid::new_v4();
    data.write().await.admin_session_id = Some(uuid);
    let claims: TokenClaims = TokenClaims {
        sub: uuid.to_string(),
        exp,
        iat,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(data.read().await.env.jwt_secret.as_ref()),
    )
    .unwrap();

    let cookie = Cookie::build(("token", token.to_owned()))
        .path("/")
        .max_age(time::Duration::hours(1))
        .same_site(SameSite::Lax)
        .http_only(true);

    let mut response = Response::new(json!({"status": "success", "token": token}).to_string());
    response
        .headers_mut()
        .insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());
    Ok(response)
}

#[tracing::instrument(name = "logout user")]
pub async fn logout_handler() -> Result<impl IntoResponse, DataApiReturn> {
    let cookie = Cookie::build(("token", ""))
        .path("/")
        .max_age(time::Duration::hours(-1))
        .same_site(SameSite::Lax)
        .http_only(true);

    let mut response = Response::new(json!({"status": "success"}).to_string());
    response
        .headers_mut()
        .insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());
    Ok(response)
}
