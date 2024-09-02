use askama::Template;
use axum::{extract::State, response::Html, Extension};
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

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
            testimonials.append(&mut builtin_testimonials());
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

fn builtin_testimonials() -> Vec<DBTestimonial> {
    vec![DBTestimonial {
        id: Uuid::new_v4(),
        firstname: "Jose".to_owned(),
        lastname: "Garcia".to_owned(),
        bio: None,
        content: r#"
I was graciously invited to attend a fly-fishing outing with a good Marine friend of mine.  All expenses were paid, and we would spend the day learning the ropes on fly fishing.  How could I say no?
<br />
<br />
We headed out to Lake Tahoe where I met Jamie Guajardo who gave us instruction on what we would be doing on our trip. I was completely surprised that Jamie, of Semper Flies Foundation, was not going to be coming with us seeing that he had arranged this entire trip through Tahoe Fly Fishing Outfitters. Thankful is not enough of a word for Jamie.  
<br />
<br />
I have been having some real bad mental health issues recently and figured that maybe this is what I needed.  And, I am glad I went. The escape from the city and just being out in the peacefulness of God's nature literally made me forget about my problems.  I spent the day learning how to fly fish with our guide, from Tahoe Fly Fishing outfitters, who was deeply knowledgeable and patient with me.  To top it off I caught a fish toward the end of the day.  
<br />
<br />
Being out there in the middle of nowhere, with the only sounds being of birds and the river water, made me forget about my problems and worries. It centered me for the day.  I am grateful for the opportunity to have attended this awesome trip and I am grateful for all involved, Jamie of Semper Flies Foundation, Tahoe Fly Fishing Outfitters and everyone else that made this day possible.  
<br />
<br />
Thank you and Semper Fidelis!"#.to_owned(),
    }]
}

fn generate_testimonials() -> Vec<DBTestimonial> {
    vec![
        DBTestimonial {
            id: uuid::Uuid::new_v4(),
            firstname: "Jane".to_string(),
            lastname: "Doe".to_string(),
            bio: Some("Widow of vet".to_string()),
            content: "I got to meet other women who struggle with what i have".to_string(),
        },
        DBTestimonial {
            id: uuid::Uuid::new_v4(),
            firstname: "Jack".to_string(),
            lastname: "Smith".to_string(),
            bio: Some("Vietnam Veteran".to_string()),
            content: "Semperflies got me in touch with other vets".to_string(),
        },
        DBTestimonial {
            id: uuid::Uuid::new_v4(),
            firstname: "John".to_string(),
            lastname: "Doe".to_string(),
            bio: None,
            content: "I love semperflies".to_string(),
        },
    ]
}
