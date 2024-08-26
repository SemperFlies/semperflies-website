use askama::Template;
use axum::{extract::State, response::Html, Extension};
use chrono::NaiveDate;
use sqlx::{Pool, Postgres};

use crate::{
    auth::middleware::SoftAuthExtension,
    components::carousel::{CarouselTemplate, Image},
    database::{handles::DbData, models::DBPatrolLog},
    state::SharedState,
};

#[derive(Template, Debug)]
#[template(path = "pages/patrol_log.html")]
pub struct PatrolLogTemplate {
    logs: Vec<Log>,
    admin: bool,
}

pub const PATROL_LOG: &str = "patrol_log";

#[derive(Debug)]
pub struct Log {
    pub id: uuid::Uuid,
    pub heading: String,
    pub description: String,
    pub date: NaiveDate,
    pub carousel: CarouselTemplate,
}

impl From<DBPatrolLog> for Log {
    fn from(value: DBPatrolLog) -> Self {
        let carousel = CarouselTemplate {
            images: value
                .img_urls
                .into_iter()
                .map(|url| Image {
                    src: url,
                    alt: String::new(),
                })
                .collect(),
        };
        Self {
            id: value.id,
            heading: value.heading,
            description: value.description,
            date: value.date,
            carousel,
        }
    }
}

async fn get_logs(pool: &Pool<Postgres>) -> anyhow::Result<Vec<Log>> {
    let dblogs = DBPatrolLog::get_multiple(pool).await?;
    Ok(dblogs.into_iter().map(|l| Log::from(l)).collect())
}

pub async fn patrol_log(
    State(data): State<SharedState>,
    Extension(soft_auth_ext): Extension<SoftAuthExtension>,
) -> Html<String> {
    let r = data.read().await;
    match get_logs(&r.db).await {
        Ok(logs) => {
            let template = PatrolLogTemplate {
                logs,
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

impl PatrolLogTemplate {
    fn render_carousel(carousel: &CarouselTemplate) -> String {
        carousel
            .render()
            .unwrap_or("error rendering carousel".to_owned())
    }
}
