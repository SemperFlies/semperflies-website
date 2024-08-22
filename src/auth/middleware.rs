use std::{str::FromStr, sync::Arc};

use super::error::{AuthError, AuthResult};
use crate::{
    auth::model::TokenClaims,
    error::{DataApiReturn, DataResponse, InternalError},
    state::SharedState,
    AppState,
};
use axum::{
    body::Body,
    extract::{FromRef, State},
    http::{header, Request},
    middleware::Next,
    response::IntoResponse,
};
use axum_extra::extract::cookie::CookieJar;
use jsonwebtoken::{decode, DecodingKey, Validation};
use tracing::{debug, warn};

#[tracing::instrument(name = "get admin session id", skip_all)]
pub fn get_admin_session_id(
    jwt_secret: &[u8],
    cookie_jar: CookieJar,
    req: &Request<Body>,
) -> AuthResult<uuid::Uuid> {
    let token = cookie_jar
        .get("token")
        .map(|cookie| cookie.value().to_string())
        .or_else(|| {
            req.headers()
                .get(header::AUTHORIZATION)
                .and_then(|auth_header| auth_header.to_str().ok())
                .and_then(|auth_value| {
                    if auth_value.starts_with("Bearer ") {
                        Some(auth_value[7..].to_owned())
                    } else {
                        None
                    }
                })
        })
        .ok_or(AuthError::NotLoggedIn)?;
    warn!("Token got from cookie jar: {:?}", token);

    let claims = decode::<TokenClaims>(
        &token,
        &DecodingKey::from_secret(jwt_secret),
        &Validation::default(),
    )?
    .claims;

    let id = uuid::Uuid::from_str(&claims.sub)?;
    Ok(id)
}

#[tracing::instrument(name = "admin authorization middleware", skip_all)]
pub async fn admin_auth(
    State(data): State<SharedState>,
    cookie_jar: CookieJar,
    req: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, DataApiReturn> {
    let r = data.read().await;
    let id = get_admin_session_id(r.env.jwt_secret.as_ref(), cookie_jar, &req)
        .map_err(|err| err.into_data_api_return())?;
    if let Some(state_id) = r.admin_session_id {
        if state_id != id {
            warn!("session id from state does not match admin id");
            return Err(AuthError::NotLoggedIn.into_data_api_return());
        }
    } else {
        warn!("no session id from state");
        return Err(AuthError::NotLoggedIn.into_data_api_return());
    }
    Ok(next.run(req).await)
}

#[derive(Clone, Debug)]
pub struct SoftAuthExtension {
    pub is_logged_in: bool,
}

#[tracing::instrument(name = "soft auth middleware", skip_all)]
pub async fn soft_auth(
    cookie_jar: CookieJar,
    State(data): State<SharedState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, DataApiReturn> {
    let r = data.read().await;
    let is_logged_in = {
        let mut ret = false;
        if let Some(user_id) =
            get_admin_session_id(r.env.jwt_secret.as_ref(), cookie_jar, &req).ok()
        {
            if let Some(session_id) = r.admin_session_id {
                warn!("got session id from state");
                ret = user_id == session_id;
            }
        }
        ret
    };
    req.extensions_mut()
        .insert(SoftAuthExtension { is_logged_in });

    Ok(next.run(req).await)
}
