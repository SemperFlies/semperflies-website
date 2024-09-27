use axum::{
    extract::Host,
    handler::HandlerWithoutStateExt,
    http::{StatusCode, Uri},
    response::Redirect,
    BoxError,
};
use axum_server::tls_rustls::RustlsConfig;
use std::{
    fs::{self, File},
    net::SocketAddr,
    path::PathBuf,
};
use tracing::info;

#[derive(Clone, Copy)]
pub struct Ports {
    pub http: u16,
    pub https: u16,
}

pub async fn get_cert_config() -> RustlsConfig {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let main_path = PathBuf::from(manifest.to_owned());
    let entries = fs::read_dir(main_path.clone()).unwrap();
    info!("entries: {entries:?}");

    let certs_dir = PathBuf::from(manifest).join("certifications");
    info!("certs directory: {certs_dir:?}");
    RustlsConfig::from_pem_file(certs_dir.join("cert.pem"), certs_dir.join("key.pem"))
        .await
        .unwrap()
}

pub async fn redirect_http_to_https(ports: Ports) {
    fn make_https(host: String, uri: Uri, ports: Ports) -> Result<Uri, BoxError> {
        let mut parts = uri.into_parts();

        parts.scheme = Some(axum::http::uri::Scheme::HTTPS);

        if parts.path_and_query.is_none() {
            parts.path_and_query = Some("/".parse().unwrap());
        }

        let https_host = host.replace(&ports.http.to_string(), &ports.https.to_string());
        parts.authority = Some(https_host.parse()?);

        Ok(Uri::from_parts(parts)?)
    }

    let redirect = move |Host(host): Host, uri: Uri| async move {
        match make_https(host, uri, ports) {
            Ok(uri) => Ok(Redirect::permanent(&uri.to_string())),
            Err(error) => {
                tracing::warn!(%error, "failed to convert URI to HTTPS");
                Err(StatusCode::BAD_REQUEST)
            }
        }
    };

    let addr = SocketAddr::from(([127, 0, 0, 1], ports.http));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, redirect.into_make_service())
        .await
        .unwrap();
}
