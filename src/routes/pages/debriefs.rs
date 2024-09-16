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
    let jose_garcia = DBTestimonial {
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
    };

    let lawrence_turner = DBTestimonial {
        id: Uuid::new_v4(),
        firstname: "Lawrence".to_owned(),
        lastname: "Turner".to_owned(),
        bio: None,
        content: r#"
        To whoever is out there thinking of trying the fishing trip with Semper Flies and Lake Tahoe Fly Fishing, I
highly recommend.
<br/>
<br/>
Some of us Veterans have experienced unfathomable things overseas that live with us day in and day
out that are unexplainable that would just not make sense, if we attempted to put into words.
<br/>
<br/>
Jamie Guajardo is a Special Forces Marine, we did not serve together but chewed the same dirt at the
same time, he is a Giant! And a special person trying to heal his brothers.
<br/>
<br/>
Long story short, Jamie knows what it is like to have the feeling that lives with us. It’s an amazing thing
what he is doing for us on this level to try to heal.
Jamie set us up with South Lake Tahoe Fly Fishing for a beautiful day outdoors to help heal and figure
out our damage and wounds.
<br/>
<br/>
Started our day off at the shop where all of the staff were Awesome! Headed out to the river where we
trekked in about a 15-20 min ride on a brand new side by side with amazing views on the way in,
beautiful water, and unspeakable experience.
<br/>
<br/>
Our guide was very knowledgeable, patient, and put us on fun fighting fish! I really appreciate what
Jamie and South Lake Tahoe Fly Fishing Shop are doing for Veterans. It’s one step closer to normalcy!
Lol!
It’s only for a day please go drop a line with Semper Flies and South Lake Tahoe Fly Fishing Shop!
<br/>
<br/>
Semper Fidelis!
<br/>
3/5 Kilo Co.
<br/>
Phantom Fury.
        "#.to_owned(),
    };
    vec![jose_garcia, lawrence_turner]
}
