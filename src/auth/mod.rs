use std::sync::LazyLock;

pub mod error;
pub mod handlers;
pub mod middleware;
pub mod model;

pub struct AdminCredentials {
    password: String,
}

pub static ADMIN_CREDENTIALS: LazyLock<AdminCredentials> = LazyLock::new(|| {
    let password = std::env::var("ADMIN_PASSWORD").expect("Failed to get admin password");
    AdminCredentials { password }
});
