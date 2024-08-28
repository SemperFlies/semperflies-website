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
        Ok(mut testimonials) => {
            testimonials.append(&mut generate_testimonials());
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

fn generate_testimonials() -> Vec<DBTestimonial> {
    let image_urls = vec![
        "public/assets/images/board_members/business.jpg".to_string(),
        "public/assets/images/board_members/business2.jpg".to_string(),
        "public/assets/images/board_members/old.jpg".to_string(),
    ];
    vec![
        DBTestimonial {
            id: uuid::Uuid::new_v4(),
            firstname: "Jane".to_string(),
            lastname: "Doe".to_string(),
            bio: Some("Widow of vet".to_string()),
            content: "I got to meet other women who struggle with what i have".to_string(),
            // image: Some(image_urls[0].clone()),
        },
        DBTestimonial {
            id: uuid::Uuid::new_v4(),
            firstname: "Jack".to_string(),
            lastname: "Smith".to_string(),
            bio: Some("Vietnam Veteran".to_string()),
            content: "Semperflies got me in touch with other vets".to_string(),
            // image: None,
        },
        DBTestimonial {
            id: uuid::Uuid::new_v4(),
            firstname: "John".to_string(),
            lastname: "Doe".to_string(),
            bio: None,
            content: "I love semperflies".to_string(),
            // image: Some(image_urls[1].clone()),
        },
    ]
}
