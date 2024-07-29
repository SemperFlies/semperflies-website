use askama::Template;
use axum::{response::Html, Extension};

use crate::auth::middleware::SoftAuthExtension;

#[derive(Template, Debug)]
#[template(path = "pages/login_logout.html")]
pub struct LoginLogoutTemplate {
    logged_in: bool,
}

#[tracing::instrument(name = "login logout ")]
pub async fn login_logout(Extension(soft_auth_ext): Extension<SoftAuthExtension>) -> Html<String> {
    let tmpl = LoginLogoutTemplate {
        logged_in: soft_auth_ext.is_logged_in,
    };
    match tmpl.render() {
        Ok(r) => Html(r),
        Err(err) => Html(format!("Error rendering Layout: {}", err.to_string())),
    }
}

#[derive(Template, Debug)]
#[template(path = "admin/upload.html")]
pub struct UploadTemplate;

pub async fn upload() -> Html<String> {
    match UploadTemplate.render() {
        Ok(r) => Html(r),
        Err(err) => Html(format!("Error rendering Layout: {}", err.to_string())),
    }
}
