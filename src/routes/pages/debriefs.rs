use askama::Template;
use axum::response::Html;
use serde::Deserialize;

#[derive(Template, Debug)]
#[template(path = "pages/debriefs.html")]
pub struct DebriefsTemplate {
    testimonials: Vec<Testimonial>,
}

#[derive(Debug)]
pub struct Testimonial {
    pub firstname: String,
    pub lastname: String,
    pub bio: Option<String>,
    pub content: String,
}
pub async fn debriefs() -> Html<String> {
    let template = DebriefsTemplate {
        testimonials: generate_testimonials(),
    };
    match template.render() {
        Ok(r) => Html(r),
        Err(err) => Html(format!("Error rendering Layout: {}", err.to_string())),
    }
}

fn generate_testimonials() -> Vec<Testimonial> {
    let image_urls = vec![
        "public/assets/images/board_members/business.jpg".to_string(),
        "public/assets/images/board_members/business2.jpg".to_string(),
        "public/assets/images/board_members/old.jpg".to_string(),
    ];
    vec![
        Testimonial {
            firstname: "Jane".to_string(),
            lastname: "Doe".to_string(),
            bio: Some("Widow of vet".to_string()),
            content: "I got to meet other women who struggle with what i have".to_string(),
            // image: Some(image_urls[0].clone()),
        },
        Testimonial {
            firstname: "Jack".to_string(),
            lastname: "Smith".to_string(),
            bio: Some("Vietnam Veteran".to_string()),
            content: "Semperflies got me in touch with other vets".to_string(),
            // image: None,
        },
        Testimonial {
            firstname: "John".to_string(),
            lastname: "Doe".to_string(),
            bio: None,
            content: "I love semperflies".to_string(),
            // image: Some(image_urls[1].clone()),
        },
    ]
}
