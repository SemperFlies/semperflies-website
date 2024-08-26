use askama::Template;
use axum::{extract::State, response::Html, Extension};
use serde::Deserialize;
use sqlx::{Pool, Postgres};

use crate::{
    auth::middleware::SoftAuthExtension,
    database::{handles::DbData, models::DBTestimonial},
    state::SharedState,
};

#[derive(Template, Debug)]
#[template(path = "pages/debriefs.html")]
pub struct DebriefsTemplate {
    testimonials: Vec<DBTestimonial>,
    admin: bool,
}

pub const DEBRIEFS: &str = "debriefs";

async fn get_testimonials(pool: &Pool<Postgres>) -> anyhow::Result<Vec<DBTestimonial>> {
    let dbtests = DBTestimonial::get_multiple(pool).await?;
    Ok(dbtests)
}

pub async fn debriefs(
    State(data): State<SharedState>,
    Extension(soft_auth_ext): Extension<SoftAuthExtension>,
) -> Html<String> {
    let r = data.read().await;

    match get_testimonials(&r.db).await {
        Ok(testimonials) => {
            let template = DebriefsTemplate {
                testimonials,
                admin: soft_auth_ext.is_logged_in,
            };

            match template.render() {
                Ok(r) => Html(r),
                Err(err) => Html(format!("Error rendering Layout: {}", err.to_string())),
            }
        }
        Err(err) => Html(format!("A database error occured: {:?}", err)),
    }
}
