use askama::Template;
use axum::response::Html;
use chrono::NaiveDate;
use rand::prelude::*;

use crate::components::carousel::{CarouselTemplate, Image};

#[derive(Template, Debug)]
#[template(path = "pages/patrol_logs.html")]
pub struct PatrolLogsTemplate {
    activities: Vec<Activity>,
}

#[derive(Debug)]
struct Activity {
    heading: String,
    description: String,
    date: NaiveDate,
    carousel: CarouselTemplate,
}

pub async fn patrol_logs() -> Html<String> {
    let template = PatrolLogsTemplate {
        activities: generate_activities(10),
    };

    match template.render() {
        Ok(r) => Html(r),
        Err(err) => Html(format!("Error rendering Layout: {}", err.to_string())),
    }
}

impl PatrolLogsTemplate {
    fn render_carousel(carousel: &CarouselTemplate) -> String {
        carousel
            .render()
            .unwrap_or("error rendering carousel".to_owned())
    }
}

fn generate_activities(amt: i32) -> Vec<Activity> {
    let mut rng = thread_rng();
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
            images.push(Image::new(&image_urls[i], "alt"))
        }
        let carousel = CarouselTemplate { images };

        activities.push(Activity {
            heading,
            description,
            date,
            carousel,
        });
    }
    activities
}
