use askama::Template;
use axum::{
    extract::{Query, State},
    response::Html,
    Extension,
};
use chrono::NaiveDate;
use rand::Rng;
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use tracing::warn;
use uuid::Uuid;

use crate::{
    auth::middleware::SoftAuthExtension,
    components::carousel::{CarouselTemplate, HasCarousel, Image},
    database::models::{DBImage, DBPatrolLog, DBPatrolLogParams},
    routes::pages::util,
    state::SharedState,
};

#[derive(Template, Debug)]
#[template(path = "pages/patrol_log.html")]
pub struct PatrolLogTemplate {
    logs: Vec<Log>,
    admin: bool,
}

#[derive(Template, Debug)]
#[template(path = "components/single_patrol_log.html")]
pub struct SinglePatrolLogTemplate {
    log: Log,
    admin: bool,
}

pub const PATROL_LOG: &str = "patrol_log";

#[derive(Debug, Clone)]
pub struct Log {
    pub id: uuid::Uuid,
    pub heading: String,
    pub description: String,
    pub date: NaiveDate,
    pub carousel: CarouselTemplate,
}
impl HasCarousel for PatrolLogTemplate {}

impl From<(DBPatrolLog, Vec<DBImage>)> for Log {
    fn from((log, images): (DBPatrolLog, Vec<DBImage>)) -> Self {
        let images: Vec<Image> = images
            .into_iter()
            .filter_map(|i| {
                if log.img_ids.contains(&i.id) {
                    Some(i.into())
                } else {
                    None
                }
            })
            .collect();

        let carousel = CarouselTemplate {
            show_subtitles: false,
            auto_scroll: false,
            images,
        };
        Self {
            id: log.id,
            heading: log.heading,
            description: log.description,
            date: log.date,
            carousel,
        }
    }
}

async fn get_logs(pool: &Pool<Postgres>) -> anyhow::Result<Vec<Log>> {
    let all_logs_and_imgs =
        DBImage::get_multiple_with_images::<DBPatrolLog, DBPatrolLogParams>(&pool).await?;
    let mut all_logs = vec![];
    for (log, imgs) in all_logs_and_imgs {
        all_logs.push(Log::from((log, imgs)));
    }
    Ok(all_logs)
}

#[derive(Debug, Deserialize)]
pub struct PatrolLogQuery {
    log_heading: Option<String>,
}

#[tracing::instrument(name = "patrol log template rendering", skip_all)]
pub async fn patrol_log(
    State(data): State<SharedState>,
    queries: Option<Query<PatrolLogQuery>>,
    Extension(soft_auth_ext): Extension<SoftAuthExtension>,
) -> Html<String> {
    let r = data.read().await;
    let mut logs_map = HashMap::new();

    for l in get_logs(&r.db).await.unwrap() {
        logs_map.insert(l.heading.to_owned(), l);
    }

    for l in generate_activities(5) {
        logs_map.insert(l.heading.to_owned(), l);
    }

    for l in builtin_logs() {
        logs_map.insert(l.heading.to_owned(), l);
    }

    if let Some(q) = queries {
        warn!("got query: {:?}", q);
        if let Some(heading) = q.0.log_heading {
            match logs_map.get(&heading) {
                Some(log) => {
                    warn!("building template for log: {:?}", log);
                    let template = Some(SinglePatrolLogTemplate {
                        log: log.clone(),
                        admin: soft_auth_ext.is_logged_in,
                    });

                    return match template.unwrap().render() {
                        Ok(r) => Html(r),
                        Err(err) => Html(format!("Error rendering Layout: {}", err.to_string())),
                    };
                }
                None => return Html(format!("{} is not a valid log heading", heading)),
            }
        }
    }

    let logs = logs_map.into_values().collect();
    let template = Some(PatrolLogTemplate {
        logs,
        admin: soft_auth_ext.is_logged_in,
    });

    match template.unwrap().render() {
        Ok(r) => Html(r),
        Err(err) => Html(format!("Error rendering Layout: {}", err.to_string())),
    }
}

fn builtin_logs() -> Vec<Log> {
    let images =
        util::all_images_in_directory("public/assets/images/patrol_log/fishing_trip").unwrap();
    let images = images
        .into_iter()
        .map(|path| Image {
            src: path.to_str().unwrap().to_string(),
            alt: String::new(),
            subtitle: String::new(),
        })
        .collect();

    let carousel = CarouselTemplate {
        show_subtitles: false,
        images,
        auto_scroll: false,
    };
    let fishing_trip = Log {
        id: Uuid::new_v4(),
        heading: "Semperflies Fishing Trip".to_string(),
        description: "Semper Flies Foundation & Tahoe Fly Fishing Outfitters teamed up to send (2) Combat Veterans on a fly fishing trip they would remember for the rest of their lives.".to_string(),
        date: NaiveDate::from_ymd_opt(2023, 06, 21).unwrap(),
        carousel,
    };
    vec![fishing_trip]
}

fn generate_activities(amt: i32) -> Vec<Log> {
    let mut rng = rand::thread_rng();
    let image_urls = vec![
        "public/assets/images/board_members/business.jpg".to_string(),
        "public/assets/images/board_members/business2.jpg".to_string(),
        "public/assets/images/board_members/old.jpg".to_string(),
    ];

    let mut activities = Vec::new();
    for _ in 0..amt {
        let heading = format!("Heading for activity {}", activities.len() + 1);
        let description = format!("Description of activity {}", activities.len() + 1);
        let year = rng.gen_range(1950..1990);
        let month = rng.gen_range(1..13);
        let day = rng.gen_range(1..29);
        let date = NaiveDate::from_ymd_opt(year, month, day).unwrap();

        let amt_imgs = rng.gen_range(0..=3);
        let mut images = vec![];
        for i in 0..amt_imgs {
            images.push(Image {
                src: image_urls[i].to_owned(),
                alt: String::new(),
                subtitle: String::new(),
            })
        }
        let carousel = CarouselTemplate {
            show_subtitles: false,
            images,
            auto_scroll: false,
        };

        activities.push(Log {
            id: uuid::Uuid::new_v4(),
            heading,
            description,
            date,
            carousel,
        });
    }
    activities
}
