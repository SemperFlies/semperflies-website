use askama::Template;
use axum::{response::Html, Extension};
use tracing::warn;

use crate::auth::middleware::SoftAuthExtension;

#[derive(Template, Debug)]
#[template(path = "admin/login_logout.html")]
pub struct LoginLogoutTemplate {
    logged_in: bool,
}

#[tracing::instrument(name = "login logout", skip_all)]
pub async fn login_logout(Extension(soft_auth_ext): Extension<SoftAuthExtension>) -> Html<String> {
    let tmpl = LoginLogoutTemplate {
        logged_in: soft_auth_ext.is_logged_in,
    };
    warn!("admin is logged in: {}", tmpl.logged_in);
    match tmpl.render() {
        Ok(r) => Html(r),
        Err(err) => Html(format!("Error rendering Layout: {}", err.to_string())),
    }
}

#[derive(Template, Debug)]
#[template(path = "admin/upload.html")]
pub struct UploadTemplate {
    logged_in: bool,
}

pub async fn upload(Extension(soft_auth_ext): Extension<SoftAuthExtension>) -> Html<String> {
    let tmpl = UploadTemplate {
        // logged_in: soft_auth_ext.is_logged_in,
        logged_in: true,
    };
    match tmpl.render() {
        Ok(r) => Html(r),
        Err(err) => Html(format!("Error rendering Layout: {}", err.to_string())),
    }
}
